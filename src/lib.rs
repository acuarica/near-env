#![deny(warnings)]

use proc_macro::{TokenStream, TokenTree};
use proc_macro2::Span;
use proc_macro2::TokenTree::Literal;
use quote::quote;
use quote::ToTokens;
use std::fmt::Write;
use std::ops::Deref;
use syn::{
    self, Attribute, Block, Error, FnArg, ImplItem, ImplItemMethod, ItemEnum, ItemImpl, ItemTrait,
    Pat, Visibility,
};

fn is_public(method: &ImplItemMethod) -> bool {
    match method.vis {
        Visibility::Public(_) => true,
        _ => false,
    }
}

#[proc_macro_derive(PanicMessage, attributes(panic_msg))]
pub fn near_panic(item: TokenStream) -> TokenStream {
    if let Ok(input) = syn::parse::<ItemEnum>(item) {
        let name = &input.ident;

        let mut cases = Vec::new();
        for var in &input.variants {
            let var_name = &var.ident;

            let mut pattern = Vec::new();
            let msg_format = if let Some(msg) = get_panic_msg(&var.attrs) {
                msg
            } else {
                return compile_error("`panic_msg` missing on `enum` variant");
            };
            for field in &var.fields {
                if let Some(field_name) = &field.ident {
                    pattern.push(quote! { #field_name });
                }
            }
            cases.push(quote! {
                #name::#var_name { #(#pattern),* } => format!(#msg_format, #(#pattern),* )
            });
        }

        (quote! {
            impl #name {
                fn msg(&self) -> String {
                    match self {
                        #(#cases),*
                    }
                }

                fn panic(self) -> ! {
                    #[derive(Serialize)]
                    #[serde(crate = "::near_sdk::serde")]
                    struct PanicResponse {
                        #[serde(flatten)]
                        panic: #name,
                        msg: String,
                    }

                    ::near_sdk::env::panic(
                        ::near_sdk::serde_json::to_string(&PanicResponse {
                            msg: self.msg(),
                            panic: self,
                        })
                        .unwrap()
                        .as_bytes(),
                    )
                }
            }
        })
        .into()
    } else {
        compile_error("`near_panic` can only be used on `enum` sections")
    }
}

#[proc_macro_attribute]
pub fn panic_msg(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn near_log(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(mut input) = syn::parse::<ItemImpl>(item) {
        let attr_args = attr.into_iter().collect::<Vec<TokenTree>>();
        let skip_args = contains_attr_arg(&attr_args, "skip_args");
        let only_pub = contains_attr_arg(&attr_args, "only_pub");

        for impl_item in input.items.iter_mut() {
            if let ImplItem::Method(method) = impl_item {
                if !only_pub || is_public(method) || input.trait_.is_some() {
                    make_loggable_fn(method, skip_args);
                }
            }
        }
        (quote! { #input }).into()
    } else {
        compile_error("`near_envlog` can only be used on `impl` sections")
    }
}

fn contains_attr_arg(attr_args: &Vec<TokenTree>, attr_arg: &str) -> bool {
    for token in attr_args {
        if let TokenTree::Ident(ident) = token {
            if ident.to_string() == attr_arg {
                return true;
            }
        }
    }

    false
}

/// `near_envlog_skip_args` is a marker attribute, it does not generate code by itself.
/// If `near_envlog` is enabled on this function, `near_envlog_skip_args` omits its arguments.
#[proc_macro_attribute]
pub fn near_envlog_skip_args(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn has_attr(attrs: &Vec<Attribute>, attr_name: &str) -> Option<usize> {
    let mut i = 0;
    for attr in attrs {
        let attr_str = attr.path.to_token_stream().to_string();
        if attr_str.ends_with(attr_name) {
            return Some(i);
        }

        i += 1;
    }
    None
}

fn make_loggable_fn(method: &mut ImplItemMethod, skip_args: bool) {
    let has_attr = |attr_name| crate::has_attr(&method.attrs, attr_name).is_some();

    let name = method.sig.ident.to_string();
    let mut is_mut = false;
    let mut args = ArgsFormatter::new(&name);

    if !skip_args && !has_attr("near_envlog_skip_args") {
        write!(args.fmt, "(").unwrap();
        for arg in method.sig.inputs.iter() {
            match arg {
                FnArg::Receiver(r) => {
                    is_mut = r.mutability.is_some();
                }
                FnArg::Typed(pat_type) => {
                    if let Pat::Ident(pat_ident) = pat_type.pat.deref() {
                        let arg_ident = &pat_ident.ident;
                        args.push_arg(arg_ident.to_string(), quote! {#arg_ident});
                    }
                }
            }
        }
        write!(args.fmt, ")").unwrap();
    }

    if is_mut {
        args.push_arg("pred", quote! { ::near_sdk::env::predecessor_account_id() });
    }
    if has_attr("payable") {
        args.push_arg("deposit", quote! { ::near_sdk::env::attached_deposit() });
    }
    if has_attr("init") || (name == "default" && method.sig.inputs.is_empty()) {
        args.push("v", quote! { env!("CARGO_PKG_VERSION") });
    }

    let fmt = args.fmt;
    let args = args.args;
    let block = &method.block;
    method.block = syn::parse::<Block>(TokenStream::from(quote! {
        {
            ::near_sdk::env::log(format!(#fmt, #(#args),*).as_bytes());
            #block
        }
    }))
    .unwrap();
}

struct ArgsFormatter {
    args: Vec<proc_macro2::TokenStream>,
    fmt: String,
}

impl ArgsFormatter {
    fn new<S: AsRef<str>>(method_name: S) -> Self {
        Self {
            args: Vec::new(),
            fmt: method_name.as_ref().to_string(),
        }
    }

    fn push_arg<S: AsRef<str>>(&mut self, name: S, value: proc_macro2::TokenStream) {
        self.push(format!("{}: ", name.as_ref()), value);
    }

    fn push<S: AsRef<str>>(&mut self, prefix: S, value: proc_macro2::TokenStream) {
        if self.fmt.ends_with(")") {
            write!(self.fmt, " ").unwrap();
        } else if !self.fmt.ends_with("(") {
            write!(self.fmt, ", ").unwrap();
        }
        self.args.push(value);
        write!(self.fmt, "{}{{}}", prefix.as_ref()).unwrap();
    }
}

fn get_panic_msg(attrs: &Vec<Attribute>) -> Option<String> {
    for attr in attrs {
        if attr.path.is_ident("panic_msg") {
            for token in attr.tokens.clone() {
                if let Literal(lit) = token {
                    if let Some(line) = lit
                        .to_string()
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                    {
                        return Some(line.trim().to_string());
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_attribute]
pub fn near_ext(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(input) = syn::parse::<ItemTrait>(item) {
        if let Some(i) = has_attr(&input.attrs, "ext_contract") {
            let mut dup = input.clone();
            dup.attrs.remove(i);
            (quote! {
               #input
               #dup
            })
            .into()
        } else {
            compile_error("`near_ext` must be used before `ext_contract`")
        }
    } else {
        compile_error("`near_ext` can only be used on `trait` sections")
    }
}

fn compile_error(message: &str) -> TokenStream {
    TokenStream::from(Error::new(Span::call_site(), message).to_compile_error())
}

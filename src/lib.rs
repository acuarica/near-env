use proc_macro::{TokenStream, TokenTree};
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use std::fmt::Write;
use std::ops::Deref;
use syn::{self, Attribute, Block, FnArg, ImplItem, ImplItemMethod, ItemImpl, Pat, Visibility};

#[proc_macro_attribute]
pub fn near_envlog(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(mut input) = syn::parse::<ItemImpl>(item) {
        let attr_args = attr.into_iter().collect::<Vec<TokenTree>>();
        let skip_args = contains_attr_arg(&attr_args, "skip_args");
        let only_pub = contains_attr_arg(&attr_args, "only_pub");

        for impl_item in input.items.iter_mut() {
            if let ImplItem::Method(method) = impl_item {
                if !only_pub
                    || match method.vis {
                        Visibility::Public(_) => true,
                        _ => false,
                    }
                {
                    make_loggable_fn(method, skip_args);
                }
            }
        }
        (quote! { #input }).into()
    } else {
        TokenStream::from(
            syn::Error::new(
                Span::call_site(),
                "`near_envlog` can only be used on `impl` sections",
            )
            .to_compile_error(),
        )
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

fn make_loggable_fn(method: &mut ImplItemMethod, skip_args: bool) {
    let name = method.sig.ident.to_string();
    let mut is_mut = false;
    let mut args = ArgsFormatter::new(&name);

    if !skip_args && !has_attr("near_envlog_skip_args", &method.attrs) {
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
    if has_attr("payable", &method.attrs) {
        args.push_arg("deposit", quote! { ::near_sdk::env::attached_deposit() });
    }
    if has_attr("init", &method.attrs) || (name == "default" && method.sig.inputs.is_empty()) {
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

fn has_attr(attr_name: &str, attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        let attr_str = attr.path.to_token_stream().to_string();
        if attr_str.ends_with(attr_name) {
            return true;
        }
    }

    false
}

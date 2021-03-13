use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use std::ops::Deref;
use std::{fmt::Write, ops::DerefMut};
use syn::{self, Attribute, Block, FnArg, ImplItem, ItemFn, ItemImpl, Pat, Signature, Visibility};

#[proc_macro_attribute]
pub fn near_envlog(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = attr.to_string();
    let should_skip_args = attr_args.contains("skip_args");
    let should_only_pub = attr_args.contains("only_pub");

    if let Ok(mut input) = syn::parse::<ItemFn>(item.clone()) {
        make_loggable_fn(
            &input.sig,
            input.block.deref_mut(),
            !should_skip_args && !skip_args(&input.attrs),
            is_payable(&input.attrs),
        );
        (quote! { #input }).into()
    } else if let Ok(mut input) = syn::parse::<ItemImpl>(item) {
        for impl_item in input.items.iter_mut() {
            if let ImplItem::Method(method) = impl_item {
                if !should_only_pub || is_public(&method.vis) {
                    make_loggable_fn(
                        &method.sig,
                        &mut method.block,
                        !should_skip_args && !skip_args(&method.attrs),
                        is_payable(&method.attrs),
                    );
                }
            }
        }
        (quote! { #input }).into()
    } else {
        TokenStream::from(
            syn::Error::new(
                Span::call_site(),
                "near_envlog can only be used on function declarations and impl sections",
            )
            .to_compile_error(),
        )
    }
}

/// `near_envlog_skip_args` is a marker attribute, it does not generate code by itself.
/// If `near_envlog` is enabled on this function, `near_envlog_skip_args` omits its arguments.
#[proc_macro_attribute]
pub fn near_envlog_skip_args(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn make_loggable_fn(sig: &Signature, block: &mut Block, with_args: bool, is_payable: bool) {
    let mut is_mut = false;
    let mut args = ArgsFormatter::new();

    if with_args {
        write!(args.fmt, "(").unwrap();
        for arg in sig.inputs.iter() {
            match arg {
                FnArg::Receiver(r) => {
                    is_mut = r.mutability.is_some();
                }
                FnArg::Typed(pat_type) => {
                    if let Pat::Ident(pat_ident) = pat_type.pat.deref() {
                        let arg_ident = &pat_ident.ident;
                        args.push(arg_ident.to_string(), quote! {#arg_ident});
                    }
                }
            }
        }
        write!(args.fmt, ")").unwrap();
    }

    if is_mut {
        args.push("pred", quote! { ::near_sdk::env::predecessor_account_id() });
    }
    if is_payable {
        args.push("deposit", quote! { ::near_sdk::env::attached_deposit() });
    }

    let mut log_str = sig.ident.to_string();
    write!(log_str, "{}", args.fmt).unwrap();
    let args = args.args;
    *block = syn::parse::<Block>(TokenStream::from(quote! {
        {
            ::near_sdk::env::log(format!(#log_str, #(#args),*).as_bytes());
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
    fn new() -> Self {
        Self {
            args: Vec::new(),
            fmt: String::new(),
        }
    }

    fn push<S: AsRef<str>>(&mut self, name: S, value: proc_macro2::TokenStream) {
        if !self.args.is_empty() {
            write!(self.fmt, ", ").unwrap();
        }
        write!(self.fmt, "{}: {{}}", name.as_ref()).unwrap();
        self.args.push(value);
    }
}

fn skip_args(attrs: &Vec<Attribute>) -> bool {
    has_attr("near_envlog_skip_args", attrs)
}

fn is_payable(attrs: &Vec<Attribute>) -> bool {
    has_attr("payable", attrs)
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

fn is_public(vis: &Visibility) -> bool {
    match vis {
        Visibility::Public(_) => true,
        _ => false,
    }
}

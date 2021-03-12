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

fn make_loggable_fn(sig: &Signature, block: &mut Block, with_args: bool) {
    let mut log_str = sig.ident.to_string();

    let mut is_mut = false;
    let mut args = Vec::new();
    if with_args {
        let mut log_args = String::new();
        for arg in sig.inputs.iter() {
            match arg {
                FnArg::Receiver(r) => {
                    is_mut = r.mutability.is_some();
                }
                FnArg::Typed(pat_type) => {
                    if let Pat::Ident(pat_ident) = pat_type.pat.deref() {
                        let arg_ident = &pat_ident.ident;
                        args.push(arg_ident);
                        if !log_args.is_empty() {
                            write!(log_args, ", ").unwrap();
                        }
                        write!(log_args, "{}: {{}}", arg_ident.to_string()).unwrap();
                    }
                }
            }
        }
        write!(log_str, "({})", log_args).unwrap();
    }

    let env_log = quote! { near_sdk::env::log };
    let env_pred = quote! { near_sdk::env::predecessor_account_id };

    let log_stmt = if is_mut {
        write!(log_str, " pred: {{}}").unwrap();
        if args.is_empty() {
            quote! {
                #env_log(format!(#log_str, #env_pred()).as_bytes());
            }
        } else {
            quote! {
                #env_log(format!(#log_str, #(#args),*, #env_pred()).as_bytes());
            }
        }
    } else {
        quote! {
            #env_log(format!(#log_str, #(#args),*).as_bytes());
        }
    };

    *block = syn::parse::<Block>(TokenStream::from(quote! {
        {
            #log_stmt
            #block
        }
    }))
    .unwrap();
}

fn skip_args(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        let attr_str = attr.path.to_token_stream().to_string();
        if attr_str.ends_with("near_envlog_skip_args") {
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

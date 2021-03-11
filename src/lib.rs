use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::ops::Deref;
use std::{fmt::Write, ops::DerefMut};
use syn::{self, Block, FnArg, ImplItem, ItemFn, ItemImpl, Pat, Signature};

#[proc_macro_attribute]
pub fn near_envlog(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(mut input) = syn::parse::<ItemFn>(item.clone()) {
        make_loggable_fn(&input.sig, input.block.deref_mut());
        (quote! { #input }).into()
    } else if let Ok(mut input) = syn::parse::<ItemImpl>(item) {
        for impl_item in input.items.iter_mut() {
            if let ImplItem::Method(method) = impl_item {
                make_loggable_fn(&method.sig, &mut method.block);
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

fn make_loggable_fn(sig: &Signature, block: &mut Block) {
    let mut log_args = String::new();
    let mut args = Vec::new();
    let mut is_mut = false;
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
                    write!(log_args, "{}: `{{:?}}`", arg_ident.to_string()).unwrap();
                }
            }
        }
    }

    let env_log = quote! { ::near_sdk::env::log };
    let env_pred = quote! { ::near_sdk::env::predecessor_account_id };

    let mut log_str = String::new();
    write!(log_str, "{}({})", sig.ident.to_string(), log_args).unwrap();
    let log_stmt = if is_mut {
        write!(log_str, " pred: `{{}}`").unwrap();
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

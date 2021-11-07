//! Shamir's secret-sharing macro

extern crate proc_macro;

use darling;
use darling::FromMeta;
use syn;
use syn::parse_macro_input;
use proc_macro2::*;
use std::collections::HashMap;
use quote::quote;
use std::iter::FromIterator;
use crate::common::*;

// template files are relative to the current file
const SHAMIR_TEMPLATE: &'static str = include_str!("../../templates/shamir.rs");


#[derive(Debug, FromMeta)]
struct ShamirArgs {
    #[darling(default)]
    gf: Option<syn::Path>,
    #[darling(default)]
    rng: Option<ExprWrapper>,
}

pub fn shamir(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = crate_path();

    // parse args
    let raw_args = parse_macro_input!(args as AttributeArgsWrapper).0;
    let args = match ShamirArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ItemMod);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let shamir = ty.ident;

    let __gf = Ident::new(&format!("__{}_gf", shamir.to_string()), Span::call_site());
    let __rng = Ident::new(&format!("__{}_rng", shamir.to_string()), Span::call_site());

    // overrides in parent's namespace
    let mut overrides = vec![];
    match args.gf.as_ref() {
        Some(gf) => {
            overrides.push(quote! {
                use #gf as #__gf;
            })
        }
        None => {
            overrides.push(quote! {
                // If not overridden, we need to create our own type in Barret mode here
                // in order to ensure the finite-field operations are constant-time
                #[#__crate::macros::gf(polynomial=0x11d, generator=0x02, barret)]
                type #__gf;
            })
        }
    }
    match args.rng.as_ref() {
        Some(ExprWrapper(rng)) => {
            overrides.push(quote! {
                #[inline]
                fn #__rng() -> impl #__crate::internal::rand::Rng {
                    #rng
                }
            })
        }
        None => {
            overrides.push(quote! {
                #[inline]
                fn #__rng() -> impl #__crate::internal::rand::Rng {
                    #__crate::internal::rand::rngs::ThreadRng::default()
                }
            })
        }
    }

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__shamir".to_owned(), TokenTree::Ident(shamir.clone())),
        ("__gf".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__gf }
        }))),
        ("__rng".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__rng }
        }))),
        ("__crate".to_owned(), __crate.clone()),
    ]);

    // parse template
    let template = match compile_template(SHAMIR_TEMPLATE, &replacements) {
        Ok(template) => template,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let output = quote! {
        #(#attrs)* #vis mod #shamir {
            #template
        }

        // overrides in parent's namespace
        #(#overrides)*
    };

    output.into()
}

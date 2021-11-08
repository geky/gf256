//! RAID-parity struct macro

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
const RAID_TEMPLATE: &'static str = include_str!("../../templates/raid.rs");


#[derive(Debug, FromMeta)]
struct RaidArgs {
    parity: u8,
    #[darling(default)]
    gf: Option<syn::Path>,
}

pub fn raid(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = crate_path();

    // parse args
    let raw_args = parse_macro_input!(args as AttributeArgsWrapper).0;
    let args = match RaidArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    // only up to 2 parity blocks are currently supported
    assert!(args.parity <= 2);

    // parse type
    let ty = parse_macro_input!(input as syn::ItemStruct);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let raid = ty.ident;

    let __mod = Ident::new(&format!("__{}_gen", raid.to_string()), Span::call_site());
    let __gf = Ident::new(&format!("__{}_gf", raid.to_string()), Span::call_site());

    // overrides in parent's namespace
    let mut overrides = vec![];
    match args.gf.as_ref() {
        Some(gf) => {
            overrides.push(quote! {
                use #gf as #__gf;
            });
        }
        None => {
            overrides.push(quote! {
                use #__crate::gf::gf256 as #__gf;
            });
        }
    }

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__raid".to_owned(), TokenTree::Ident(raid.clone())),
        ("__gf".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__gf }
        }))),
        ("__parity".to_owned(), TokenTree::Literal(
            Literal::u8_unsuffixed(args.parity)
        )),
        ("__crate".to_owned(), __crate.clone()),
    ]);

    // parse template
    let template = match compile_template(RAID_TEMPLATE, &replacements) {
        Ok(template) => template,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let output = quote! {
        #(#attrs)* #vis use #__mod::#raid;
        mod #__mod {
            #template
        }

        // overrides in parent's namespace
        #(#overrides)*
    };

    output.into()
}

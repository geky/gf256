//! LFSR struct macro

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
const LFSR_TEMPLATE: &'static str = include_str!("../../templates/lfsr.rs");


#[derive(Debug, FromMeta)]
struct LfsrArgs {
    gf: syn::Path,
    u: syn::Path,
}

pub fn lfsr(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = crate_path();

    // parse args
    let raw_args = parse_macro_input!(args as AttributeArgsWrapper).0;
    let args = match LfsrArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ItemStruct);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let lfsr = ty.ident;

    let __mod = Ident::new(&format!("__{}_gen", lfsr.to_string()), Span::call_site());
    let __gf = Ident::new(&format!("__{}_gf", lfsr.to_string()), Span::call_site());
    let __u = Ident::new(&format!("__{}_u", lfsr.to_string()), Span::call_site());

    // overrides in parent's namespace
    let mut overrides = vec![];
    let gf = args.gf;
    overrides.push(quote! {
        use #gf as #__gf;
    });
    let u = args.u;
    overrides.push(quote! {
        use #u as #__u;
    });

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__lfsr".to_owned(), TokenTree::Ident(lfsr.clone())),
        ("__gf".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__gf }
        }))),
        ("__u".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__u }
        }))),
        ("__crate".to_owned(), __crate.clone()),
    ]);

    // parse template
    let template = match compile_template(LFSR_TEMPLATE, &replacements) {
        Ok(template) => template,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let output = quote! {
        #(#attrs)* #vis use #__mod::#lfsr;
        mod #__mod {
            #template
        }

        // overrides in parent's namespace
        #(#overrides)*
    };

    output.into()
}

//! Reed-Solomon error-correction macro

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
const RS_TEMPLATE: &'static str = include_str!("../templates/rs.rs");


#[derive(Debug, FromMeta)]
struct RsArgs {
    block: usize,
    data: usize,

    #[darling(default)]
    gf: Option<syn::Path>,
    #[darling(default)]
    u: Option<syn::Path>,
}

pub fn rs(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = crate_path();

    // parse args
    let raw_args = parse_macro_input!(args as AttributeArgsWrapper).0;
    let args = match RsArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    // gf256 is limited to 255 elements
    assert!(args.block <= 255);
    assert!(args.data <= args.block);

    // parse type
    let ty = parse_macro_input!(input as syn::ItemMod);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let rs = ty.ident;

    let __gf = Ident::new(&format!("__{}_gf", rs.to_string()), Span::call_site());
    let __u  = Ident::new(&format!("__{}_u",  rs.to_string()), Span::call_site());

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
                use #__crate::gf::gf256 as #__gf;
            })
        }
    }
    match args.u.as_ref() {
        Some(u) => {
            overrides.push(quote! {
                use #u as #__u;
            })
        }
        None => {
            // default to u8, we can't do any better since we don't really have
            // a way to infer the underlying u-type of __gf
            //
            // we could use an inherent associated type in __gf, except they are
            // currently not supported
            // https://github.com/rust-lang/rust/issues/8995
            //
            overrides.push(quote! {
                use u8 as #__u;
            });
        }
    }

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__rs".to_owned(), TokenTree::Ident(rs.clone())),
        ("__block_size".to_owned(), TokenTree::Literal(
            Literal::usize_unsuffixed(args.block)
        )),
        ("__data_size".to_owned(), TokenTree::Literal(
            Literal::usize_unsuffixed(args.data)
        )),
        ("__ecc_size".to_owned(), TokenTree::Literal(
            Literal::usize_unsuffixed(args.block-args.data)
        )),
        ("__gf".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__gf }
        }))),
        ("__u".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__u }
        }))),
        ("__crate".to_owned(), __crate.clone()),
    ]);

    // parse template
    let template = match compile_template(RS_TEMPLATE, &replacements) {
        Ok(template) => template,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let output = quote! {
        #(#attrs)* #vis mod #rs {
            #template
        }

        // overrides in parent's namespace
        #(#overrides)*
    };

    output.into()
}

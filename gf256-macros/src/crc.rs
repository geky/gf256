//! CRC function macro

extern crate proc_macro;

use darling;
use darling::FromMeta;
use syn;
use syn::parse_macro_input;
use proc_macro2::*;
use std::collections::HashMap;
use quote::quote;
use quote::ToTokens;
use std::iter::FromIterator;
use std::convert::TryFrom;
use std::cmp::max;
use crate::common::*;

// template files are relative to the current file
const CRC_TEMPLATE: &'static str = include_str!("../../templates/crc.rs");


#[derive(Debug, FromMeta)]
struct CrcArgs {
    polynomial: U128,

    #[darling(default)]
    u: Option<String>,
    #[darling(default)]
    width: Option<usize>,
    #[darling(default)]
    p: Option<syn::Path>,
    #[darling(default)]
    p2: Option<syn::Path>,

    #[darling(default)]
    reversed: Option<bool>,
    #[darling(default)]
    inverted: Option<bool>,

    #[darling(default)]
    naive: bool,
    #[darling(default)]
    table: bool,
    #[darling(default)]
    small_table: bool,
    #[darling(default)]
    barret: bool,
}

pub fn crc(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = crate_path();

    // parse args
    let raw_args = parse_macro_input!(args as syn::AttributeArgs);
    let args = match CrcArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let width = match args.width {
        Some(width) => width,
        None => {
            // default to 1 less than the width of the given polynomial, this
            // is the only width that would really work
            let polynomial = args.polynomial.0;
            (128-usize::try_from(polynomial.leading_zeros()).unwrap()) - 1
        }
    };
    assert!(width == width.next_power_of_two() && width >= 8);

    let u = match args.u.as_ref() {
        Some(u) => u.clone(),
        None => format!("u{}", max(width.next_power_of_two(), 8)),
    };
    let u2 = format!("u{}", 2*max(width.next_power_of_two(), 8));

    let p = match args.p.as_ref() {
        Some(p) => TokenTree::Group(Group::new(Delimiter::None, p.to_token_stream())),
        None => TokenTree::Group(Group::new(Delimiter::None, {
            let p = TokenTree::Ident(Ident::new(&format!("p{}", max(width.next_power_of_two(), 8)), Span::call_site()));
            quote! { #__crate::p::#p }
        }))
    };
    let p2 = match args.p2.as_ref() {
        Some(p2) => TokenTree::Group(Group::new(Delimiter::None, p2.to_token_stream())),
        None => TokenTree::Group(Group::new(Delimiter::None, {
            let p2 = TokenTree::Ident(Ident::new(&format!("p{}", 2*max(width.next_power_of_two(), 8)), Span::call_site()));
            quote! { #__crate::p::#p2 }
        }))
    };

    // decide between implementations
    let (naive, table, small_table, barret) = match
        (args.naive, args.table, args.small_table, args.barret)
    {
        // choose mode if one is explicitly requested
        (true,  false, false, false) => (true,  false, false, false),
        (false, true,  false, false) => (false, true,  false, false),
        (false, false, true,  false) => (false, false, true,  false),
        (false, false, false, true ) => (false, false, false, true ),

        (false, false, false, false) => {
            // if xmul is available, Barret reduction is the fastest option for
            // CRCs, otherwise a table-based approach wins
            let input = TokenStream::from(input);
            let xmul = xmul_predicate();
            let output = quote! {
                #[cfg_attr(#xmul,      #__crate::macros::crc(barret, #(#raw_args),*))]
                #[cfg_attr(not(#xmul), #__crate::macros::crc(table,  #(#raw_args),*))]
                #input
            };
            return output.into();
        },

        // multiple modes selected?
        _ => panic!("invalid configuration of macro crc (naive, table, small_table, barret?)"),
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ForeignItemFn);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let crc = ty.sig.ident;

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__crc".to_owned(), TokenTree::Ident(crc.clone())),
        ("__polynomial".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed(args.polynomial.0)
        )),
        ("__u".to_owned(), TokenTree::Ident(
            Ident::new(&u, Span::call_site())
        )),
        ("__u2".to_owned(), TokenTree::Ident(
            Ident::new(&u2, Span::call_site())
        )),
        ("__width".to_owned(), TokenTree::Literal(
            Literal::usize_unsuffixed(width)
        )),
        ("__is_usize".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", u == "usize"), Span::call_site())
        )),
        ("__p".to_owned(), p),
        ("__p2".to_owned(), p2),
        ("__reversed".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", args.reversed.unwrap_or(true)), Span::call_site())
        )),
        ("__inverted".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", args.inverted.unwrap_or(true)), Span::call_site())
        )),
        ("__naive".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", naive), Span::call_site())
        )),
        ("__table".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", table), Span::call_site())
        )),
        ("__small_table".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", small_table), Span::call_site())
        )),
        ("__barret".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", barret), Span::call_site())
        )),
        ("__crate".to_owned(), __crate),
    ]);

    // parse template
    let template = match compile_template(CRC_TEMPLATE, &replacements) {
        Ok(template) => template,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let crcmod = Ident::new(&format!("__{}_gen", crc.to_string()), Span::call_site());
    let output = quote! {
        #(#attrs)* #vis use #crcmod::#crc;
        mod #crcmod {
            #template
        }
    };

    output.into()
}

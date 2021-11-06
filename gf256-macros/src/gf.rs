//! Galois-field type macro

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
const GF_TEMPLATE: &'static str = include_str!("../../templates/gf.rs");


#[derive(Debug, FromMeta)]
struct GfArgs {
    polynomial: U128,
    generator: u64,

    #[darling(default)]
    u: Option<String>,
    #[darling(default)]
    width: Option<usize>,
    #[darling(default)]
    p: Option<syn::Path>,
    #[darling(default)]
    p2: Option<syn::Path>,

    #[darling(default)]
    naive: bool,
    #[darling(default)]
    log_table: bool,
    #[darling(default)]
    rem_table: bool,
    #[darling(default)]
    small_rem_table: bool,
    #[darling(default)]
    barret: bool,
}

pub fn gf(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = crate_path();

    // parse args
    let raw_args = parse_macro_input!(args as syn::AttributeArgs);
    let args = match GfArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let width = match args.width {
        Some(width) => width,
        None => {
            // default to 1 less than the width of the irreducible polynomial
            // that defines the field, since, well, this is actually the only
            // width that would work with that polynomial
            let polynomial = args.polynomial.0;
            (128-usize::try_from(polynomial.leading_zeros()).unwrap()) - 1
        }
    };

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
    let (naive, log_table, rem_table, small_rem_table, barret) = match
        (args.naive, args.log_table, args.rem_table, args.small_rem_table, args.barret)
    {
        // choose mode if one is explicitly requested
        (true,  false, false, false, false) => (true,  false, false, false, false),
        (false, true,  false, false, false) => (false, true,  false, false, false),
        (false, false, true,  false, false) => (false, false, true,  false, false),
        (false, false, false, true , false) => (false, false, false, true , false),
        (false, false, false, false, true ) => (false, false, false, false, true ),

        // if width <= 8, default to log_table as this is currently the fastest
        // implementation, but uses O(2^n) memory
        (false, false, false, false, false) if width <= 8 => (false, true, false, false, false),

        // otherwise it turns out Barret reduction is the fastest, even when
        // carry-less multiplication isn't available
        (false, false, false, false, false) => (false, false, false, false, true),

        // multiple modes selected?
        _ => panic!("invalid configuration of macro gf (naive, log_table, rem_table, small_rem_table, barret?)"),
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ForeignItemType);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let gf = ty.ident;

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__gf".to_owned(), TokenTree::Ident(gf.clone())),
        ("__polynomial".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed(args.polynomial.0)
        )),
        ("__generator".to_owned(), TokenTree::Literal(
            Literal::u64_unsuffixed(args.generator)
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
        ("__nonzeros".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed((1u128 << width) - 1)
        )),
        ("__is_pw2ge8".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", width.is_power_of_two() && width >= 8), Span::call_site())
        )),
        ("__is_usize".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", u == "usize"), Span::call_site())
        )),
        ("__p".to_owned(), p),
        ("__p2".to_owned(), p2),
        ("__naive".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", naive), Span::call_site())
        )),
        ("__log_table".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", log_table), Span::call_site())
        )),
        ("__rem_table".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", rem_table), Span::call_site())
        )),
        ("__small_rem_table".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", small_rem_table), Span::call_site())
        )),
        ("__barret".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", barret), Span::call_site())
        )),
        ("__crate".to_owned(), __crate),
    ]);

    // parse template
    let template = match compile_template(GF_TEMPLATE, &replacements) {
        Ok(template) => template,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let gfmod = Ident::new(&format!("__{}_gen", gf.to_string()), Span::call_site());
    let output = quote! {
        #(#attrs)* #vis use #gfmod::#gf;
        mod #gfmod {
            #template
        }
    };

    output.into()
}

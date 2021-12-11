//! Galois-field type macro

extern crate proc_macro;

use darling;
use darling::FromMeta;
use syn;
use syn::parse_macro_input;
use proc_macro2::*;
use std::collections::HashMap;
use quote::quote;
use std::iter::FromIterator;
use std::convert::TryFrom;
use std::cmp::max;
use crate::common::*;

// template files are relative to the current file
const GF_TEMPLATE: &'static str = include_str!("../templates/gf.rs");


#[derive(Debug, FromMeta)]
struct GfArgs {
    polynomial: U128Wrapper,
    generator: u64,

    #[darling(default, rename="usize")]
    is_usize: Option<bool>,
    #[darling(default)]
    u: Option<syn::Path>,
    #[darling(default)]
    u2: Option<syn::Path>,
    #[darling(default)]
    p: Option<syn::Path>,
    #[darling(default)]
    p2: Option<syn::Path>,

    #[darling(default)]
    naive: bool,
    #[darling(default)]
    table: bool,
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
    let raw_args = parse_macro_input!(args as AttributeArgsWrapper).0;
    let args = match GfArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let width = {
        // default to 1 less than the width of the irreducible polynomial
        // that defines the field, since, well, this is actually the only
        // width that would work with that polynomial
        let polynomial = args.polynomial.0;
        (128-usize::try_from(polynomial.leading_zeros()).unwrap()) - 1
    };

    let is_usize = match args.is_usize {
        Some(is_usize) => is_usize,
        None => {
            match args.u.as_ref().and_then(guess_is_usize) {
                Some(is_usize) => is_usize,
                // assume u is not usize by default, since this is more common
                None => false,
            }
        }
    };

    // decide between implementations
    let (naive, table, rem_table, small_rem_table, barret) = match
        (args.naive, args.table, args.rem_table, args.small_rem_table, args.barret)
    {
        // choose mode if one is explicitly requested
        (true,  false, false, false, false) => (true,  false, false, false, false),
        (false, true,  false, false, false) => (false, true,  false, false, false),
        (false, false, true,  false, false) => (false, false, true,  false, false),
        (false, false, false, true , false) => (false, false, false, true , false),
        (false, false, false, false, true ) => (false, false, false, false, true ),

        // if no-tables/small-tables are enabled, stick to Barret reduction as
        // it is only beaten by the 2x256-byte log-tables
        (false, false, false, false, false)
            if cfg!(any(feature="no-tables", feature="small-tables"))
            => (false, false, false, false, true),

        // if width <= 8, default to table as this is currently the fastest
        // implementation, but uses O(2^n) memory
        (false, false, false, false, false)
            if width <= 8
            => (false, true, false, false, false),

        // otherwise it turns out Barret reduction is the fastest, even when
        // carry-less multiplication isn't available
        (false, false, false, false, false) => (false, false, false, false, true),

        // multiple modes selected?
        _ => panic!("invalid configuration of macro gf (naive, table, rem_table, small_rem_table, barret?)"),
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ForeignItemType);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let gf = ty.ident;

    let __mod = Ident::new(&format!("__{}_gen", gf.to_string()), Span::call_site());
    let __u   = Ident::new(&format!("__{}_u",   gf.to_string()), Span::call_site());
    let __u2  = Ident::new(&format!("__{}_u2",  gf.to_string()), Span::call_site());
    let __p   = Ident::new(&format!("__{}_p",   gf.to_string()), Span::call_site());
    let __p2  = Ident::new(&format!("__{}_p2",  gf.to_string()), Span::call_site());

    // overrides in paren't namespace
    let mut overrides = vec![];
    match args.u.as_ref() {
        Some(u) => {
            overrides.push(quote! {
                use #u as #__u;
            })
        }
        None => {
            let u = Ident::new(&format!("u{}", max(width.next_power_of_two(), 8)), Span::call_site());
            overrides.push(quote! {
                use #u as #__u;
            })
        }
    }
    match args.u2.as_ref() {
        Some(u2) => {
            overrides.push(quote! {
                use #u2 as #__u2;
            })
        }
        None => {
            let u2 = Ident::new(&format!("u{}", 2*max(width.next_power_of_two(), 8)), Span::call_site());
            overrides.push(quote! {
                use #u2 as #__u2;
            })
        }
    }
    match args.p.as_ref() {
        Some(p) => {
            overrides.push(quote! {
                use #p as #__p;
            })
        }
        None => {
            let p = Ident::new(&format!("p{}", max(width.next_power_of_two(), 8)), Span::call_site());
            overrides.push(quote! {
                use #__crate::p::#p as #__p;
            })
        }
    }
    match args.p2.as_ref() {
        Some(p2) => {
            overrides.push(quote! {
                use #p2 as #__p2;
            })
        }
        None => {
            let p2 = Ident::new(&format!("p{}", 2*max(width.next_power_of_two(), 8)), Span::call_site());
            overrides.push(quote! {
                use #__crate::p::#p2 as #__p2;
            })
        }
    }

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__gf".to_owned(), TokenTree::Ident(gf.clone())),
        ("__polynomial".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed(args.polynomial.0)
        )),
        ("__generator".to_owned(), TokenTree::Literal(
            Literal::u64_unsuffixed(args.generator)
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
            Ident::new(&format!("{}", is_usize), Span::call_site())
        )),
        ("__u".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__u }
        }))),
        ("__u2".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__u2 }
        }))),
        ("__p".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__p }
        }))),
        ("__p2".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__p2 }
        }))),
        ("__naive".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", naive), Span::call_site())
        )),
        ("__table".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", table), Span::call_site())
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

    let output = quote! {
        #(#attrs)* #vis use #__mod::#gf;
        mod #__mod {
            #template
        }

        // overrides in parent's namespace
        #(#overrides)*
    };

    output.into()
}

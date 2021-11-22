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
use std::cmp::max;
use std::convert::TryFrom;
use crate::common::*;

// template files are relative to the current file
const LFSR_TEMPLATE: &'static str = include_str!("../templates/lfsr.rs");


#[derive(Debug, FromMeta)]
struct LfsrArgs {
    polynomial: U128Wrapper,

    #[darling(default)]
    u: Option<syn::Path>,
    #[darling(default)]
    u2: Option<syn::Path>,
    #[darling(default)]
    nzu: Option<syn::Path>,
    #[darling(default)]
    nzu2: Option<syn::Path>,
    #[darling(default)]
    p: Option<syn::Path>,
    #[darling(default)]
    p2: Option<syn::Path>,

    #[darling(default)]
    reflected: Option<bool>,

    // div/rem modes
    #[darling(default)]
    naive: bool,
    #[darling(default)]
    table: bool,
    #[darling(default)]
    small_table: bool,
    #[darling(default)]
    barret: bool,
    #[darling(default)]
    table_barret: bool,
    #[darling(default)]
    small_table_barret: bool,

    // skip modes
    #[darling(default)]
    naive_skip: bool,
    #[darling(default)]
    table_skip: bool,
    #[darling(default)]
    small_table_skip: bool,
    #[darling(default)]
    barret_skip: bool,
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

    let width = {
        // default to 1 less than the width of the given polynomial, this
        // is the only width that would really work
        let polynomial = args.polynomial.0;
        (128-usize::try_from(polynomial.leading_zeros()).unwrap()) - 1
    };

    // decide between div/rem modes
    let (naive, table, small_table, barret, table_barret, small_table_barret) = match
        (args.naive, args.table, args.small_table, args.barret, args.table_barret, args.small_table_barret)
    {
        // choose mode if one is explicitly requested
        (true,  false, false, false, false, false) => (true,  false, false, false, false, false),
        (false, true,  false, false, false, false) => (false, true,  false, false, false, false),
        (false, false, true,  false, false, false) => (false, false, true,  false, false, false),
        (false, false, false, true,  false, false) => (false, false, false, true,  false, false),
        (false, false, false, false, true,  false) => (false, false, false, false, true,  false),
        (false, false, false, false, false, true ) => (false, false, false, false, false, true ),

        // if no-tables is enabled, naive is actually the fastest (Barret
        // reduction behaves uniquely terrible for LFSRs for some reason,
        // though Barret reduction for skipping is still the fastest)
        (false, false, false, false, false, false)
            if cfg!(feature="no-tables")
            => (true,  false, false, false, false, false),

        // if small-tables is enabled, we can use a smaller 16-element table
        (false, false, false, false, false, false)
            if cfg!(feature="small-tables")
            => (false, false, true,  false, false, false),

        // otherwise tables is the fastest
        (false, false, false, false, false, false)
            => (false, true,  false, false, false, false),

        // multiple modes selected?
        _ => panic!("invalid configuration of macro lfsr (naive, table, small_table, barret, table_barret, small_table_barret?)"),
    };

    // decide between skip modes
    let (naive_skip, table_skip, small_table_skip, barret_skip) = match
        (args.naive_skip, args.table_skip, args.small_table_skip, args.barret_skip)
    {
        // choose mode if one is explicitly requested
        (true,  false, false, false) => (true,  false, false, false),
        (false, true,  false, false) => (false, true,  false, false),
        (false, false, true,  false) => (false, false, true,  false),
        (false, false, false, true ) => (false, false, false, true ),

        // otherwise default to Barret reduction, this is the fastest across
        // all options, even when hardware xmul isn't available
        //
        // We know this from evaluation of gf-types, since skipping is just
        // finite-field multiplication. See gf.rs for more info.
        //
        (false, false, false, false) => (false, false, false, true),

        // multiple modes selected?
        _ => panic!("invalid configuration of macro lfsr (naive_skip, table_skip, small_table_skip, barret_skip)"),
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ItemStruct);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let lfsr = ty.ident;

    let __mod  = Ident::new(&format!("__{}_gen",  lfsr.to_string()), Span::call_site());
    let __u    = Ident::new(&format!("__{}_u",    lfsr.to_string()), Span::call_site());
    let __u2   = Ident::new(&format!("__{}_u2",   lfsr.to_string()), Span::call_site());
    let __nzu  = Ident::new(&format!("__{}_nzu",  lfsr.to_string()), Span::call_site());
    let __nzu2 = Ident::new(&format!("__{}_nzu2", lfsr.to_string()), Span::call_site());
    let __p    = Ident::new(&format!("__{}_p",    lfsr.to_string()), Span::call_site());
    let __p2   = Ident::new(&format!("__{}_p2",   lfsr.to_string()), Span::call_site());

    // overrides in parent's namespace
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
    match args.nzu.as_ref() {
        Some(nzu) => {
            overrides.push(quote! {
                use #nzu as #__nzu;
            })
        }
        None => {
            let nzu = Ident::new(&format!("NonZeroU{}", max(width.next_power_of_two(), 8)), Span::call_site());
            overrides.push(quote! {
                use core::num::#nzu as #__nzu;
            })
        }
    }
    match args.nzu2.as_ref() {
        Some(nzu2) => {
            overrides.push(quote! {
                use #nzu2 as #__nzu2;
            })
        }
        None => {
            let nzu2 = Ident::new(&format!("NonZeroU{}", 2*max(width.next_power_of_two(), 8)), Span::call_site());
            overrides.push(quote! {
                use core::num::#nzu2 as #__nzu2;
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
        ("__lfsr".to_owned(), TokenTree::Ident(lfsr.clone())),
        ("__polynomial".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed(args.polynomial.0)
        )),
        ("__inverse_polynomial".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed(args.polynomial.0.reverse_bits() >> args.polynomial.0.leading_zeros())
        )),
        ("__width".to_owned(), TokenTree::Literal(
            Literal::usize_unsuffixed(width)
        )),
        ("__nonzeros".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed((1u128 << width) - 1)
        )),
        ("__u".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__u }
        }))),
        ("__u2".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__u2 }
        }))),
        ("__nzu".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__nzu }
        }))),
        ("__nzu2".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__nzu2 }
        }))),
        ("__p".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__p }
        }))),
        ("__p2".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__p2 }
        }))),
        ("__reflected".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", args.reflected.unwrap_or(false)), Span::call_site())
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
        ("__table_barret".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", table_barret), Span::call_site())
        )),
        ("__small_table_barret".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", small_table_barret), Span::call_site())
        )),
        ("__naive_skip".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", naive_skip), Span::call_site())
        )),
        ("__table_skip".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", table_skip), Span::call_site())
        )),
        ("__small_table_skip".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", small_table_skip), Span::call_site())
        )),
        ("__barret_skip".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", barret_skip), Span::call_site())
        )),
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

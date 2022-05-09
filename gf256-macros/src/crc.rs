//! CRC function macro

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
const CRC_TEMPLATE: &'static str = include_str!("../templates/crc.rs");


#[derive(Debug, FromMeta)]
struct CrcArgs {
    polynomial: U128Wrapper,

    #[darling(default)]
    u: Option<syn::Path>,
    #[darling(default)]
    u2: Option<syn::Path>,
    #[darling(default)]
    p: Option<syn::Path>,
    #[darling(default)]
    p2: Option<syn::Path>,

    #[darling(default)]
    reflected: Option<bool>,
    #[darling(default)]
    xor: Option<U128Wrapper>,

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
    let raw_args = parse_macro_input!(args as AttributeArgsWrapper).0;
    let args = match CrcArgs::from_list(&raw_args) {
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

    // decide between implementations
    let (naive, table, small_table, barret) = match
        (args.naive, args.table, args.small_table, args.barret)
    {
        // choose mode if one is explicitly requested
        (true,  false, false, false) => (true,  false, false, false),
        (false, true,  false, false) => (false, true,  false, false),
        (false, false, true,  false) => (false, false, true,  false),
        (false, false, false, true ) => (false, false, false, true ),

        // if no-tables is enabled, stick to Barret reduction, it beats
        // a naive implementation even without hardware xmul
        (false, false, false, false)
            if cfg!(feature="no-tables")
            => (false, false, false, true),

        // if small-tables is enabled, we can use a smaller 16-element table
        (false, false, false, false)
            if cfg!(feature="small-tables")
            => {
            // if xmul is available, Barret reduction is the fastest option for
            // CRCs, otherwise a table-based approach wins
            let input = TokenStream::from(input);
            let xmul = xmul_predicate();
            let output = quote! {
                #[cfg_attr(#xmul,      #__crate::crc::crc(barret,      #(#raw_args),*))]
                #[cfg_attr(not(#xmul), #__crate::crc::crc(small_table, #(#raw_args),*))]
                #input
            };
            return output.into();
        }

        (false, false, false, false) => {
            // if xmul is available, Barret reduction is the fastest option for
            // CRCs, otherwise a table-based approach wins
            let input = TokenStream::from(input);
            let xmul = xmul_predicate();
            let output = quote! {
                #[cfg_attr(#xmul,      #__crate::crc::crc(barret, #(#raw_args),*))]
                #[cfg_attr(not(#xmul), #__crate::crc::crc(table,  #(#raw_args),*))]
                #input
            };
            return output.into();
        },

        // multiple modes selected?
        _ => panic!("invalid configuration of macro crc (naive, table, small_table, barret?)"),
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ItemFn);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let crc = ty.sig.ident;

    let __mod = Ident::new(&format!("__{}_gen", crc.to_string()), Span::call_site());
    let __u   = Ident::new(&format!("__{}_u",   crc.to_string()), Span::call_site());
    let __u2  = Ident::new(&format!("__{}_u2",  crc.to_string()), Span::call_site());
    let __p   = Ident::new(&format!("__{}_p",   crc.to_string()), Span::call_site());
    let __p2  = Ident::new(&format!("__{}_p2",  crc.to_string()), Span::call_site());

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
        ("__crc".to_owned(), TokenTree::Ident(crc.clone())),
        ("__polynomial".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed(args.polynomial.0)
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
        ("__p".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__p }
        }))),
        ("__p2".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__p2 }
        }))),
        ("__reflected".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", args.reflected.unwrap_or(true)), Span::call_site())
        )),
        ("__xor".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed(
                args.xor.map(|xor| xor.0)
                    .unwrap_or_else(|| (1u128 << width) - 1)
            )
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

    let output = quote! {
        #(#attrs)* #vis use #__mod::#crc;
        mod #__mod {
            #template
        }

        // overrides in parent's namespace
        #(#overrides)*
    };

    output.into()
}

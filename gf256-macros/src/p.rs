//! Polynomial type macro

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
use crate::common::*;

// template files are relative to the current file
const P_TEMPLATE: &'static str = include_str!("../../templates/p.rs");


#[derive(Debug, FromMeta)]
struct PArgs {
    #[darling(default)]
    u: Option<String>,
    #[darling(default)]
    width: Option<usize>,

    #[darling(default)]
    naive: bool,
    #[darling(default)]
    xmul: Option<darling::util::Override<syn::Path>>,
}

pub fn p(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = crate_path();

    // parse args
    let raw_args = parse_macro_input!(args as syn::AttributeArgs);
    let args = match PArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let width = match (args.width, args.u.as_deref()) {
        (Some(width), _) => width,
        (None, Some("usize")) => {
            // annoyingly, we can't actually get the target_width in a
            // proc_macro, but we _can_ emit some boilerplate to determine
            // the target_width, and recurse back into our proc_macro.
            //
            // terrible? yes, but it works
            //
            let input = TokenStream::from(input);
            let output = quote! {
                #[cfg_attr(target_pointer_width="8",   #__crate::macros::p(width=8,   #(#raw_args),*))]
                #[cfg_attr(target_pointer_width="16",  #__crate::macros::p(width=16,  #(#raw_args),*))]
                #[cfg_attr(target_pointer_width="32",  #__crate::macros::p(width=32,  #(#raw_args),*))]
                #[cfg_attr(target_pointer_width="64",  #__crate::macros::p(width=64,  #(#raw_args),*))]
                #[cfg_attr(target_pointer_width="128", #__crate::macros::p(width=128, #(#raw_args),*))]
                #input
            };
            return output.into();
        }
        (None, Some(u)) => {
            u.get(1..)
                .and_then(|s| s.parse::<usize>().ok())
                .expect("unknown type for \"u\"")
        }
        _ => panic!("no width or u specified"),
    };

    let u = match args.u.as_ref() {
        Some(u) => u.clone(),
        None => format!("u{}", width),
    };
    let i = format!("i{}", &u[1..]);

    // decide between implementations
    let has_xmul = match (args.naive, args.xmul.as_ref()) {
        (true, None) => false,
        (false, Some(_)) => true,
        (false, None) => {
            // query target configuration and recurse back into our proc_macro
            let input = TokenStream::from(input);
            let xmul = xmul_predicate();
            let output = quote! {
                #[cfg_attr(#xmul,      #__crate::macros::p(xmul,  #(#raw_args),*))]
                #[cfg_attr(not(#xmul), #__crate::macros::p(naive, #(#raw_args),*))]
                #input
            };
            return output.into();
        },

        // multiple modes selected?
        _ => panic!("invalid configuration of macro p (naive, hardware?)"),
    };

    let xmul = match args.xmul.as_ref() {
        Some(darling::util::Override::Explicit(xmul)) => {
            TokenTree::Group(Group::new(Delimiter::None, xmul.to_token_stream()))
        }
        _ => {
            TokenTree::Group(Group::new(Delimiter::None, {
                let xmul = TokenTree::Ident(Ident::new(&format!("xmul{}", width), Span::call_site()));
                quote! { #__crate::internal::xmul::#xmul }
            }))
        }
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ForeignItemType);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let p = ty.ident;

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__p".to_owned(), TokenTree::Ident(p.clone())),
        ("__u".to_owned(), TokenTree::Ident(
            Ident::new(&u, Span::call_site())
        )),
        ("__i".to_owned(), TokenTree::Ident(
            Ident::new(&i, Span::call_site())
        )),
        ("__width".to_owned(), TokenTree::Literal(
            Literal::usize_unsuffixed(width)
        )),
        ("__is_usize".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", u == "usize"), Span::call_site())
        )),
        ("__has_xmul".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", has_xmul), Span::call_site())
        )),
        ("__xmul".to_owned(), xmul),
        ("__crate".to_owned(), __crate),
    ]);

    // parse template
    let template = match compile_template(P_TEMPLATE, &replacements) {
        Ok(template) => template,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let pmod = Ident::new(&format!("__{}_gen", p.to_string()), Span::call_site());
    let output = quote! {
        #(#attrs)* #vis use #pmod::#p;
        mod #pmod {
            #template
        }
    };

    output.into()
}


//! Polynomial type macro

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
const P_TEMPLATE: &'static str = include_str!("../templates/p.rs");


#[derive(Debug, FromMeta)]
struct PArgs {
    #[darling(default)]
    width: Option<usize>,
    #[darling(default, rename="usize")]
    is_usize: Option<bool>,
    #[darling(default)]
    u: Option<syn::Path>,
    #[darling(default)]
    i: Option<syn::Path>,

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
    let raw_args = parse_macro_input!(args as AttributeArgsWrapper).0;
    let args = match PArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
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

    let width = match (args.width, is_usize) {
        (Some(width), _) => width,
        (None, true) => {
            // annoyingly, we can't actually get the target_width in a
            // proc_macro, but we _can_ emit some boilerplate to determine
            // the target_width, and recurse back into our proc_macro.
            //
            // terrible? yes, but it works
            //
            let input = TokenStream::from(input);
            let output = quote! {
                #[cfg_attr(target_pointer_width="8",   #__crate::p::p(width=8,   #(#raw_args),*))]
                #[cfg_attr(target_pointer_width="16",  #__crate::p::p(width=16,  #(#raw_args),*))]
                #[cfg_attr(target_pointer_width="32",  #__crate::p::p(width=32,  #(#raw_args),*))]
                #[cfg_attr(target_pointer_width="64",  #__crate::p::p(width=64,  #(#raw_args),*))]
                #[cfg_attr(target_pointer_width="128", #__crate::p::p(width=128, #(#raw_args),*))]
                #input
            };
            return output.into();
        }
        (None, false) => {
            match args.u.as_ref().and_then(guess_width) {
                Some(width) => width,
                None => panic!("no width specified in p-macro?"),
            }
        }
    };

    // decide between implementations
    let has_xmul = match (args.naive, args.xmul.as_ref()) {
        (true, None) => false,
        (false, Some(_)) => true,
        (false, None) => {
            // query target configuration and recurse back into our proc_macro
            let input = TokenStream::from(input);
            let xmul = xmul_predicate();
            let output = quote! {
                #[cfg_attr(#xmul,      #__crate::p::p(xmul,  #(#raw_args),*))]
                #[cfg_attr(not(#xmul), #__crate::p::p(naive, #(#raw_args),*))]
                #input
            };
            return output.into();
        },

        // multiple modes selected?
        _ => panic!("invalid configuration of macro p (naive, hardware?)"),
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ForeignItemType);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let p = ty.ident;

    let __mod  = Ident::new(&format!("__{}_gen",  p.to_string()), Span::call_site());
    let __u    = Ident::new(&format!("__{}_u",    p.to_string()), Span::call_site());
    let __i    = Ident::new(&format!("__{}_i",    p.to_string()), Span::call_site());
    let __xmul = Ident::new(&format!("__{}_xmul", p.to_string()), Span::call_site());

    // overrides in paren't namespace
    let mut overrides = vec![];
    match args.u.as_ref() {
        Some(u) => {
            overrides.push(quote! {
                use #u as #__u;
            })
        }
        None => {
            let u = Ident::new(&format!("u{}", width), Span::call_site());
            overrides.push(quote! {
                use #u as #__u;
            })
        }
    }
    match args.i.as_ref() {
        Some(i) => {
            overrides.push(quote! {
                use #i as #__i;
            })
        }
        None => {
            let i = Ident::new(&format!("i{}", width), Span::call_site());
            overrides.push(quote! {
                use #i as #__i;
            })
        }
    }
    match args.xmul.as_ref() {
        Some(darling::util::Override::Explicit(xmul)) => {
            overrides.push(quote! {
                use #xmul as #__xmul;
            })
        }
        Some(darling::util::Override::Inherit) => {
            let xmul = TokenTree::Ident(Ident::new(&format!("xmul{}", width), Span::call_site()));
            overrides.push(quote! {
                use #__crate::internal::xmul::#xmul as #__xmul;
            })
        }
        None => {
            // no xmul
        }
    };

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__p".to_owned(), TokenTree::Ident(p.clone())),
        ("__width".to_owned(), TokenTree::Literal(
            Literal::usize_unsuffixed(width)
        )),
        ("__is_usize".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", is_usize), Span::call_site())
        )),
        ("__has_xmul".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", has_xmul), Span::call_site())
        )),
        ("__u".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__u }
        }))),
        ("__i".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__i }
        }))),
        ("__xmul".to_owned(), TokenTree::Group(Group::new(Delimiter::None, {
            quote! { super::#__xmul }
        }))),
        ("__crate".to_owned(), __crate),
    ]);

    // parse template
    let template = match compile_template(P_TEMPLATE, &replacements) {
        Ok(template) => template,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let output = quote! {
        #(#attrs)* #vis use #__mod::#p;
        mod #__mod {
            #template
        }

        // overrides in parent's namespace
        #(#overrides)*
    };

    output.into()
}



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
use std::env;
use std::convert::TryFrom;

// template files are relative to the current file
const P_TEMPLATE: &'static str = include_str!("../../templates/p.rs");
const GF_TEMPLATE: &'static str = include_str!("../../templates/gf.rs");


fn __crate() -> TokenTree {
    TokenTree::Group(Group::new(Delimiter::None,
        if env::var("CARGO_CRATE_NAME").unwrap() == "gf256" {
            quote! { crate }
        } else {
            quote! { ::gf256 }
        }
    ))
}

fn xmul_query() -> TokenStream {
    quote! {
        any(
            all(
                not(feature="no-xmul"),
                target_arch="x86_64",
                target_feature="pclmulqdq"
            ),
            all(
                not(feature="no-xmul"),
                feature="nightly",
                target_arch="aarch64",
                target_feature="neon"
            )
        )
    }
}

// u128 currently doesn't support darling::FromMeta
// TODO create PR upstream?
#[derive(Debug)]
struct U128(u128);

impl darling::FromMeta for U128 {
    fn from_string(s: &str) -> darling::Result<Self> {
        s.parse::<u128>()
            .map(|u| U128(u))
            .map_err(|_| darling::Error::unknown_value(s))
    }

    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        match *value {
            syn::Lit::Str(ref s) => Self::from_string(&s.value()),
            syn::Lit::Int(ref s) => Ok(U128(s.base10_parse::<u128>().unwrap())),
            _ => Err(darling::Error::unexpected_lit_type(value)),
        }
        .map_err(|e| e.with_span(value))
    }
}


fn token_replace(
    input: TokenStream,
    replacements: &HashMap<String, TokenTree>
) -> TokenStream {
    // helper function to set span recursively
    fn token_setspan(tt: &mut TokenTree, span: Span) {
        tt.set_span(span);
        if let TokenTree::Group(group) = tt {
            let mut ngroup = Group::new(
                group.delimiter(),
                group.stream().into_iter().map(|mut tt| {
                    token_setspan(&mut tt, span);
                    tt
                }).collect()
            );
            ngroup.set_span(group.span());
            *tt = TokenTree::Group(ngroup)
        }
    }

    input.into_iter()
        .map(|tt| {
            match tt {
                TokenTree::Ident(ident) => {
                    match replacements.get(&ident.to_string()) {
                        Some(to) => {
                            let mut to = to.clone();
                            token_setspan(&mut to, ident.span());
                            to
                        }
                        None => {
                            TokenTree::Ident(ident)
                        }
                    }
                },
                TokenTree::Group(group) => {
                    let mut ngroup = Group::new(
                        group.delimiter(),
                        token_replace(group.stream(), replacements),
                    );
                    ngroup.set_span(group.span());
                    TokenTree::Group(ngroup)
                },
                _ => tt,
            }
        })
        .collect()
}

fn token_if(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let mut output = Vec::new();
    let mut iter = input.into_iter();
    while let Some(tt) = iter.next() {
        match tt {
            TokenTree::Ident(ident) => {
                if ident.to_string() == "__if" {
                    // grab rest of conditional
                    let cond = match iter.next().unwrap() {
                        TokenTree::Group(group) => group,
                        _ => Err(syn::Error::new(ident.span(), "expected group?"))?,
                    };

                    // eval
                    let res = evalexpr::eval_boolean(&format!("{}", cond));

                    // output?
                    match res {
                        Ok(true) => {
                            output.extend(quote! { all() });
                        }
                        Ok(false) => {
                            output.extend(quote! { any() });
                        }
                        Err(err) => {
                            // return immediately because we can't expand
                            // compile_error in an attribute
                            Err(syn::Error::new(cond.span(), err))?;
                        }
                    }
                } else {
                    output.push(TokenTree::Ident(ident));
                }
            }
            TokenTree::Group(group) => {
                let mut ngroup = Group::new(
                    group.delimiter(),
                    token_if(group.stream())?,
                );
                ngroup.set_span(group.span());
                output.push(TokenTree::Group(ngroup));
            }
            _ => {
                output.push(tt);
            }
        }
    }

    Ok(output.into_iter().collect())
}

fn compile_template(
    template: &str,
    replacements: &HashMap<String, TokenTree>,
) -> Result<TokenStream, syn::Error> {
    // parse template into TokenStream
    let stream = template.parse::<TokenStream>()?;

    // replace replacements
    let stream = token_replace(stream, replacements);

    // evaluate conditionals
    let stream = token_if(stream)?;

    Ok(stream)
}


// Polynomial types

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

#[proc_macro_attribute]
pub fn p(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = __crate();

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
            let xmul_query = xmul_query();
            let output = quote! {
                #[cfg_attr(#xmul_query,      #__crate::macros::p(xmul,  #(#raw_args),*))]
                #[cfg_attr(not(#xmul_query), #__crate::macros::p(naive, #(#raw_args),*))]
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


// Galois field types

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
    table: bool,
    #[darling(default)]
    barret: bool,
    // TODO constant_time option?
}

#[proc_macro_attribute]
pub fn gf(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let __crate = __crate();

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
        None => format!("u{}", ((width+7)/8)*8),
    };
    let u2 = format!("u{}", 2*((width+7)/8)*8);

    let p = match args.p.as_ref() {
        Some(p) => TokenTree::Group(Group::new(Delimiter::None, p.to_token_stream())),
        None => TokenTree::Group(Group::new(Delimiter::None, {
            let p = TokenTree::Ident(Ident::new(&format!("p{}", ((width+7)/8)*8), Span::call_site()));
            quote! { #__crate::p::#p }
        }))
    };
    let p2 = match args.p2.as_ref() {
        Some(p2) => TokenTree::Group(Group::new(Delimiter::None, p2.to_token_stream())),
        None => TokenTree::Group(Group::new(Delimiter::None, {
            let p2 = TokenTree::Ident(Ident::new(&format!("p{}", 2*((width+7)/8)*8), Span::call_site()));
            quote! { #__crate::p::#p2 }
        }))
    };

    // decide between implementations
    let (naive, table, barret) = match (args.naive, args.table, args.barret) {
        // choose mode if one is explicitly requested
        (true,  false, false) => (true,  false, false),
        (false, true,  false) => (false, true,  false),
        (false, false, true ) => (false, false, true ),

        // if width <= 8, default to table as this is currently the fastest
        // implementation, but uses O(2^n) memory
        (false, false, false) if width <= 8 => (false, true, false),

        // otherwise there are two option, Barret reduction is faster when
        // carry-less multiplication is available, if not, a naive bitwise
        // implementation is actually the fastest
        (false, false, false) => {
            let input = TokenStream::from(input);
            let xmul_query = xmul_query();
            let output = quote! {
                #[cfg_attr(#xmul_query,      #__crate::macros::gf(barret, #(#raw_args),*))]
                #[cfg_attr(not(#xmul_query), #__crate::macros::gf(naive,  #(#raw_args),*))]
                #input
            };
            return output.into();
        },

        // multiple modes selected?
        _ => panic!("invalid configuration of macro gf (naive, table, barret?)"),
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
        ("__size".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed(1u128 << width)
        )),
        // this is mostly here due to typing issues, __size-1 triggers
        // underflow asserts all over the place for some reason
        ("__nonzeros".to_owned(), TokenTree::Literal(
            Literal::u128_unsuffixed((1u128 << width) - 1)
        )),
        ("__is_pw256p2".to_owned(), TokenTree::Ident(
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
        ("__table".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", table), Span::call_site())
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

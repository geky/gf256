
extern crate proc_macro;

use darling;
use darling::FromMeta;
use syn;
use syn::parse_macro_input;
use proc_macro2::*;
use std::collections::HashMap;
use std::collections::HashSet;
use quote::quote;
use std::iter::FromIterator;
use std::env;

// template files are relative to the current file
const P_TEMPLATE: &'static str = include_str!("p.rs");
const GF_TEMPLATE: &'static str = include_str!("gf.rs");


fn crate_() -> TokenTree {
    TokenTree::Group(Group::new(Delimiter::None,
        if env::var("CARGO_CRATE_NAME").unwrap() == "gf256" {
            quote! { crate }
        } else {
            quote! { ::gf256 }
        }
    ))
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
    u: String,
    #[darling(default)]
    width: Option<usize>,
    #[darling(default)]
    naive: bool,
    #[darling(default)]
    hardware: bool,
}

#[proc_macro_attribute]
pub fn p(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let crate_ = crate_();

    // parse args
    let args = parse_macro_input!(args as syn::AttributeArgs);
    let args = match PArgs::from_list(&args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    // decide between implementations
    let (naive, hardware) = match (
        (args.naive, args.hardware),
        (cfg!(feature="use-naive-xmul"), cfg!(feature="use-hardware-xmul"))
    ) {
        // choose mode if one is explicitly requested
        ((true,  false), _             ) => (true,  false),
        ((false, false), (true,  false)) => (true,  false),
        ((false, true,), _             ) => (false, true ),
        ((false, false), (false, true )) => (false, true ),

        // default to neither, let the p* implementation make the decision
        ((false, false), (false, false)) => (false, false),

        // multiple modes selected?
        _ => panic!("invalid configuration of macro p (naive, hardware?)"),
    };

    let width = match (args.width, args.u.as_ref()) {
        (Some(width), _) => width,
        (None, "usize") => {
            // annoyingly, we can't actually get the target_width in a
            // proc_macro, but we _can_ emit some boilerplate to determine
            // the target_width, and recurse back into our proc_macro.
            //
            // terrible? yes, but it works
            //
            let input = TokenStream::from(input);
            let output = quote! {
                #[cfg_attr(target_pointer_width="8",   #crate_::macros::p(u="usize", width=8,   naive=#naive, hardware=#hardware))]
                #[cfg_attr(target_pointer_width="16",  #crate_::macros::p(u="usize", width=16,  naive=#naive, hardware=#hardware))]
                #[cfg_attr(target_pointer_width="32",  #crate_::macros::p(u="usize", width=32,  naive=#naive, hardware=#hardware))]
                #[cfg_attr(target_pointer_width="64",  #crate_::macros::p(u="usize", width=64,  naive=#naive, hardware=#hardware))]
                #[cfg_attr(target_pointer_width="128", #crate_::macros::p(u="usize", width=128, naive=#naive, hardware=#hardware))]
                #input
            };
            return output.into();
        }
        (None, u) => {
            u.get(1..)
                .and_then(|s| s.parse::<usize>().ok())
                .expect("unknown type for \"u\"")
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
            Ident::new(&args.u, Span::call_site())
        )),
        ("__i".to_owned(), TokenTree::Ident(
            Ident::new(&format!("i{}", &args.u[1..]), Span::call_site())
        )),
        ("__width".to_owned(), TokenTree::Literal(
            Literal::usize_unsuffixed(width)
        )),
        ("__is_usize".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", args.u == "usize"), Span::call_site())
        )),
        ("__naive".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", naive), Span::call_site())
        )),
        ("__hardware".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", hardware), Span::call_site())
        )),
        ("__crate".to_owned(), crate_),
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
    polynomial: u16,
    #[darling(default)]
    generator: Option<u8>,
    #[darling(default)]
    u: Option<String>,
    #[darling(default)]
    width: Option<usize>,
    #[darling(default)]
    naive: bool,
    #[darling(default)]
    table: bool,
    #[darling(default)]
    barret: bool,
}

/// Carry-less multiplication, aka polynomial multiplication
///
fn xmul16(a: u16, b: u16) -> u16 {
    let mut x = 0;
    for i in 0..16 {
        let mask = (((a as i16) << (15-i)) >> 15) as u16;
        x ^= mask & (b << i);
    }
    x
}

/// Tests is a given element is a multiplicative generator
/// for a given field defined by the polynomial
///
/// This just attempts to generate all elements of the field,
/// erroring if not all elements could be generated
///
fn is_generator(polynomial: u16, generator: u8) -> bool {
    let mut elements = HashSet::with_capacity(255);

    // try to generate elements
    let mut x = 1u16;
    for _ in 0..255 {
        elements.insert(x as u8);
        x = xmul16(x, generator as u16);
        if x >= 256 {
            x = x ^ polynomial;
        }
    }

    // set contains all elements?
    (1..255).all(|e| elements.contains(&e))
}

/// Find a multiplicative generator for a given field defined
/// by the polynomial
///
/// This is just implemented by brute force, fortunately it's
/// usually pretty easy to find generators in a field
///
fn find_generator(polynomial: u16) -> Result<u8, String> {
    for generator in 2..255 {
        if is_generator(polynomial, generator) {
            return Ok(generator);
        }
    }

    Err(format!("unable to find a generator for field with \
        polynomial=0x{:03x}", polynomial))
}

#[proc_macro_attribute]
pub fn gf(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let crate_ = crate_();

    // parse args
    let args = parse_macro_input!(args as syn::AttributeArgs);
    let args = match GfArgs::from_list(&args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    // decide between implementations
    let (naive, table, barret) = match (
        (args.naive, args.table, args.barret),
        (cfg!(feature="use-naive-gfmul"), cfg!(feature="use-table-gfmul"), cfg!(feature="use-barret-gfmul"))
    ) {
        // choose mode if one is explicitly requested
        ((true,  false, false), _                    ) => (true,  false, false),
        ((false, false, false), (true,  false, false)) => (true,  false, false),
        ((false, true,  false), _                    ) => (false, true,  false),
        ((false, false, false), (false, true,  false)) => (false, true,  false),
        ((false, false, true ), _                    ) => (false, false, true ),
        ((false, false, false), (false, false, true )) => (false, false, true ),

        // default to table, this is currently the fastest implementation
        ((false, false, false), (false, false, false)) => (false, true,  false),

        // multiple modes selected?
        _ => panic!("invalid configuration of macro gf (naive, table, barret?)"),
    };

    // find a generator, or just fake it if we're using a barret implementation
    let generator = match args.generator {
        Some(generator) => generator,
        None => {
            match find_generator(args.polynomial) {
                Ok(generator) => generator,
                Err(err) => panic!("{}", err),
            }
        }
    };

    // parse type
    let ty = parse_macro_input!(input as syn::ForeignItemType);
    let attrs = ty.attrs;
    let vis = ty.vis;
    let gf = ty.ident;

    // keyword replacements
    let replacements = HashMap::from_iter([
        ("__gf".to_owned(), TokenTree::Ident(gf.clone())),
        ("__naive".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", naive), Span::call_site())
        )),
        ("__table".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", table), Span::call_site())
        )),
        ("__barret".to_owned(), TokenTree::Ident(
            Ident::new(&format!("{}", barret), Span::call_site())
        )),
        ("__polynomial".to_owned(), TokenTree::Literal(
            Literal::u16_unsuffixed(args.polynomial)
        )),
        ("__generator".to_owned(), TokenTree::Literal(
            Literal::u8_unsuffixed(generator)
        )),
        ("__crate".to_owned(), crate_),
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


extern crate proc_macro;

use darling;
use syn;
use proc_macro2::*;
use std::collections::HashMap;
use quote::quote;
use std::env;


pub(crate) fn crate_path() -> TokenTree {
    TokenTree::Group(Group::new(Delimiter::None,
        if env::var("CARGO_CRATE_NAME").unwrap() == "gf256" {
            quote! { crate }
        } else {
            quote! { ::gf256 }
        }
    ))
}

// TODO does this xmul query even work out of crate? (feature flag?)
pub(crate) fn xmul_predicate() -> TokenStream {
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
pub(crate) struct U128(pub u128);

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


pub(crate) fn token_replace(
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

pub(crate) fn token_if(input: TokenStream) -> Result<TokenStream, syn::Error> {
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

pub(crate) fn compile_template(
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



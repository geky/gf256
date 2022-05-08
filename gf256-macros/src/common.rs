
extern crate proc_macro;

use darling;
use syn;
use proc_macro2::*;
use std::collections::HashMap;
use quote::quote;
use std::env;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use quote::ToTokens;
use syn::parse::discouraged::Speculative;


pub(crate) fn crate_path() -> TokenTree {
    TokenTree::Group(Group::new(Delimiter::None,
        if env::var("CARGO_CRATE_NAME").unwrap() == "gf256" {
            quote! { crate }
        } else {
            quote! { ::gf256 }
        }
    ))
}

pub(crate) fn xmul_predicate() -> TokenStream {
    // override here since our features won't be available
    // in dependent crates
    if cfg!(feature="no-xmul") {
        quote! { any() }
    } else {
        quote! {
            any(
                all(
                    target_arch="x86_64",
                    target_feature="pclmulqdq"
                ),
                all(
                    target_arch="aarch64",
                    target_feature="neon"
                )
            )
        }
    }
}

/// Guess width of u type
pub(crate) fn guess_width(u: &syn::Path) -> Option<usize> {
    if u.segments.len() == 1 {
        let s = u.segments[0].ident.to_string();
        if s.starts_with('u') {
            if let Some(width) = s.get(1..)
                .and_then(|s| s.parse::<usize>().ok())
            {
                return Some(width);
            }
        }
    }

    None
}

/// Guess if u type is a usize
pub(crate) fn guess_is_usize(u: &syn::Path) -> Option<bool> {
    if u.segments.len() == 1 {
        let s = u.segments[0].ident.to_string();
        if s.starts_with('u') {
            return Some(s == "usize");
        }
    }

    None
}

// u128 currently doesn't support darling::FromMeta
// TODO create PR upstream?
#[derive(Debug)]
pub(crate) struct U128Wrapper(pub u128);

impl darling::FromMeta for U128Wrapper {
    fn from_string(s: &str) -> darling::Result<Self> {
        s.parse::<u128>()
            .map(|u| U128Wrapper(u))
            .map_err(|_| darling::Error::unknown_value(s))
    }

    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        match *value {
            syn::Lit::Str(ref s) => Self::from_string(&s.value()),
            syn::Lit::Int(ref s) => Ok(U128Wrapper(s.base10_parse::<u128>().unwrap())),
            _ => Err(darling::Error::unexpected_lit_type(value)),
        }
        .map_err(|e| e.with_span(value))
    }
}

// FromMeta for syn::Expr
#[derive(Debug)]
pub(crate) struct ExprWrapper(pub syn::Expr);

impl darling::FromMeta for ExprWrapper {
    fn from_string(s: &str) -> darling::Result<Self> {
        syn::parse_str(s)
            .map(ExprWrapper)
            .map_err(|_| darling::Error::unknown_value(s))
    }

    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(ref s) = *value {
            s.parse()
                .map(ExprWrapper)
                .map_err(|_| darling::Error::unexpected_lit_type(value))
        } else {
            Err(darling::Error::unexpected_lit_type(value))
        }
    }
}

// Special wrapper for AttributeArgs that converts exprs to strings so
// they can be treated as Meta arguments
#[derive(Debug)]
pub(crate) struct AttributeArgsWrapper(pub syn::AttributeArgs);

impl syn::parse::Parse for AttributeArgsWrapper {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut metas = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }

            // This is a bit of a hack to work around syn's strict requirement
            // on literals as values in meta key-value pairs
            //
            // We first try to parse as a normal meta-name-value, if that fails
            // we do a bit of manual parsing to extract an Expr, and force it
            // into a string. Hacky, but it work enough for our library.
            //
            let fork = input.fork();
            match fork.parse() {
                Ok(value) => {
                    metas.push(value);
                    input.advance_to(&fork);
                }
                Err(err) => {
                    match || -> syn::Result<(syn::Ident, syn::Token![=], syn::Expr)> {
                        Ok((
                            input.call(syn::Ident::parse_any)?,
                            input.parse()?,
                            input.parse()?,
                        ))
                    }() {
                        Ok((ident, eq, expr)) => {
                            metas.push(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                                path: syn::Path::from(ident),
                                eq_token: eq,
                                lit: syn::Lit::Str(syn::LitStr::new(
                                    &format!("{}", expr.to_token_stream()),
                                    expr.span()
                                )),
                            })))
                        }
                        Err(_) => {
                            // return original error
                            Err(err)?;
                        }
                    }
                }
            }

            if input.is_empty() {
                break;
            }
            input.parse::<syn::Token![,]>()?;
        }

        Ok(AttributeArgsWrapper(metas))
    }
}


/// Replace identifiers in token stream
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

/// Evaluated __if(blablabla) expressions, leaving either any() (false), or
/// all() (true) in the token stream. These can be used to participate in
/// cfg attributes. The expressions are evaluated using the evalexpr crate.
///
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

/// General-purpose template parser
///
/// Converts one TokenStream to another doing conversions based
/// on the provided map of idents to TokenTrees. This also evaluates
/// __if expressions, compile-time predicates, using the evalexpr
/// crate.
///
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



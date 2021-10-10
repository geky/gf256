
extern crate proc_macro;

use darling;
use darling::FromMeta;
use syn;
use syn::parse_macro_input;
use proc_macro2::*;
use std::collections::HashMap;
use quote::quote;
use std::iter::FromIterator;
use std::env;

// template files are relative to the current file
const P_TEMPLATE: &'static str = include_str!("p.rs");


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



#[derive(Debug, FromMeta)]
struct PArgs {
    u: String,
    #[darling(default)]
    width: Option<usize>,
}

#[proc_macro_attribute]
pub fn p(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let crate_ = TokenTree::Group(Group::new(Delimiter::None,
        if env::var("CARGO_CRATE_NAME").unwrap() == "gf256" {
            quote! { crate }
        } else {
            quote! { ::gf256 }
        }
    ));

    // parse args
    let args = parse_macro_input!(args as syn::AttributeArgs);
    let args = match PArgs::from_list(&args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
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
                #[cfg_attr(target_pointer_width="8",   #crate_::macros::p(u="usize", width=8  ))]
                #[cfg_attr(target_pointer_width="16",  #crate_::macros::p(u="usize", width=16 ))]
                #[cfg_attr(target_pointer_width="32",  #crate_::macros::p(u="usize", width=32 ))]
                #[cfg_attr(target_pointer_width="64",  #crate_::macros::p(u="usize", width=64 ))]
                #[cfg_attr(target_pointer_width="128", #crate_::macros::p(u="usize", width=128))]
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
    let template = quote! {
        #(#attrs)* #vis use #pmod::#p;
        mod #pmod {
            #template
        }
    };

    template.into()
}

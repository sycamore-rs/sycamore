use std::mem;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Comma, Paren};
use syn::{parenthesized, parse_quote, Expr, GenericArgument, Path, Result};

/// Components are identical to function calls.
pub struct Component {
    pub path: Path,
    pub paren: Paren,
    pub args: Punctuated<Expr, Comma>,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            path: input.parse()?,
            paren: parenthesized!(content in input),
            args: content.parse_terminated(Expr::parse)?,
        })
    }
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Component { path, paren, args } = self;
        let mut path = path.clone();

        let generic_arg: GenericArgument = parse_quote! { _ };
        let generics = mem::take(&mut path.segments.last_mut().unwrap().arguments);
        let generics = match generics {
            syn::PathArguments::None => quote! {},
            syn::PathArguments::AngleBracketed(mut generics) => {
                if !generics.args.is_empty() {
                    // Add the Html type param to generics.
                    let first_generic_param_index = generics
                        .args
                        .iter()
                        .enumerate()
                        .find(|(_, arg)| {
                            matches!(arg, GenericArgument::Type(_) | GenericArgument::Const(_))
                        })
                        .map(|(i, _)| i);
                    if let Some(first_generic_param_index) = first_generic_param_index {
                        generics.args.insert(first_generic_param_index, generic_arg);
                    } else {
                        generics.args.push(generic_arg);
                    }
                }
                generics.into_token_stream()
            }
            syn::PathArguments::Parenthesized(_) => unreachable!(),
        };

        let quoted = if args.empty_or_trailing() {
            quote_spanned! { paren.span=>
                {
                    #[allow(unused_imports)]
                    use ::sycamore::component::__InstantiateComponent;
                    #path#generics::__instantiate_component(())
                }
            }
        } else {
            quote_spanned! { path.span()=>
                {
                    #[allow(unused_imports)]
                    use ::sycamore::component::__InstantiateComponent;
                    #path#generics::__instantiate_component(#args)
                }
            }
        };

        tokens.extend(quoted);
    }
}

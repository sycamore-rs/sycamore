use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Comma, Paren};
use syn::{parenthesized, Expr, GenericArgument, Path, PathArguments, Result, Token};

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

        let component_path_segment = path.segments.last().unwrap();
        let mut component_generics = match &component_path_segment.arguments {
            PathArguments::None => Punctuated::<GenericArgument, Token![,]>::new(),
            PathArguments::AngleBracketed(generics) => generics.args.clone(),
            PathArguments::Parenthesized(_) => unreachable!(),
        };

        component_generics.push(syn::parse_quote! { _ });

        path.segments.last_mut().unwrap().arguments =
            PathArguments::AngleBracketed(syn::parse_quote! { <#component_generics> });

        let quoted = if args.empty_or_trailing() {
            quote_spanned! { paren.span=>
                ::maple_core::reactive::untrack(||
                    <#path as ::maple_core::component::Component::<_>>::create(())
                )
            }
        } else {
            quote_spanned! { path.span()=>
                ::maple_core::reactive::untrack(||
                    <#path as ::maple_core::component::Component::<_>>::create(#args)
                )
            }
        };

        tokens.extend(quoted);
    }
}

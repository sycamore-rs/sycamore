#![allow(clippy::eval_order_dependence)] // Needed when using `syn::parenthesized!`.

mod attributes;
mod children;
mod component;
mod element;
mod text;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::token::Paren;
use syn::{parse_macro_input, Ident, LitStr, Result, Token};

pub(crate) enum HtmlType {
    Component,
    Element,
    Text,
}

pub(crate) enum HtmlTree {
    Component(component::Component),
    Element(element::Element),
    Text(text::Text),
}

impl HtmlTree {
    fn peek_type(input: ParseStream) -> Option<HtmlType> {
        let input = input.fork(); // do not affect original ParseStream

        if input.peek(LitStr) || input.peek(Paren) {
            Some(HtmlType::Text)
        } else if input.peek(Token![::]) {
            Some(HtmlType::Component)
        } else if input.peek(Ident::peek_any) {
            let ident: Ident = input.call(Ident::parse_any).ok()?;
            let ident = ident.to_string();

            if ident.chars().next().unwrap().is_ascii_uppercase() || input.peek(Token![::]) {
                Some(HtmlType::Component)
            } else {
                Some(HtmlType::Element)
            }
        } else {
            None
        }
    }
}

impl Parse for HtmlTree {
    fn parse(input: ParseStream) -> Result<Self> {
        let html_type = match Self::peek_type(input) {
            Some(html_type) => html_type,
            None => return Err(input.error("expected a valid HTML node")),
        };

        Ok(match html_type {
            HtmlType::Component => Self::Component(input.parse()?),
            HtmlType::Element => Self::Element(input.parse()?),
            HtmlType::Text => Self::Text(input.parse()?),
        })
    }
}

impl ToTokens for HtmlTree {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let quoted = match self {
            Self::Component(component) => quote! {
                #component
            },
            Self::Element(element) => quote! {
                ::maple_core::template_result::TemplateResult::new_node(#element)
            },
            Self::Text(text) => match text {
                text::Text::Text(_) => quote! {
                    ::maple_core::template_result::TemplateResult::new_node(
                        ::maple_core::generic_node::GenericNode::text_node(#text),
                    )
                },
                text::Text::Splice(_, _) => unimplemented!("splice at top level is not supported"),
            },
        };

        tokens.extend(quoted);
    }
}

pub(crate) struct HtmlRoot {
    children: Vec<HtmlTree>,
}

impl Parse for HtmlRoot {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children = Vec::new();

        while !input.is_empty() {
            children.push(input.parse()?);
        }

        Ok(Self { children })
    }
}

impl ToTokens for HtmlRoot {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let quoted = match self.children.as_slice() {
            [] => quote! {
                ::maple_core::template_result::TemplateResult::empty()
            },
            [node] => node.to_token_stream(),
            nodes => quote! {
                ::maple_core::template_result::TemplateResult::new_fragment({
                    let mut children = ::std::vec::Vec::new();
                    #( for node in #nodes {
                        children.push(node);
                    } )*
                    children
                })
            },
        };

        tokens.extend(quoted);
    }
}

/// A macro for ergonomically creating complex UI structures.
///
/// TODO: write some more docs
#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as HtmlRoot);

    TokenStream::from(input.to_token_stream())
}

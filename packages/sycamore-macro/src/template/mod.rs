#![allow(clippy::eval_order_dependence)] // Needed when using `syn::parenthesized!`.

mod attributes;
mod children;
mod component;
mod element;
mod text;

use attributes::*;
use children::*;
use component::*;
use element::*;
use text::*;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::token::Paren;
use syn::{Ident, LitStr, Result, Token};

pub enum HtmlType {
    Component,
    Element,
    Text,
}

pub enum HtmlTree {
    Component(Component),
    Element(Element),
    Text(Text),
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
                ::sycamore::template::Template::new_node(#element)
            },
            Self::Text(text) => match text {
                Text::Str(_) => quote! {
                    ::sycamore::template::Template::new_node(
                        ::sycamore::generic_node::GenericNode::text_node(#text),
                    )
                },
                Text::Splice(_, _) => quote! {
                    ::sycamore::template::Template::new_lazy(move ||
                        ::sycamore::render::IntoTemplate::create(&#text)
                    )
                },
            },
        };

        tokens.extend(quoted);
    }
}

pub struct HtmlRoot {
    pub children: Vec<HtmlTree>,
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
                ::sycamore::template::Template::empty()
            },
            [node] => node.to_token_stream(),
            nodes => quote! {
                ::sycamore::template::Template::new_fragment({
                    let mut children = ::std::vec::Vec::new();
                    #(
                        children.push(#nodes);
                    )*
                    children
                })
            },
        };

        tokens.extend(quoted);
    }
}

pub fn template_impl(component: HtmlRoot) -> TokenStream {
    component.to_token_stream()
}

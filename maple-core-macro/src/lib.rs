mod attributes;
mod children;
mod component;
mod element;
mod text;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, Result, Token};

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

        if input.peek(Token![#]) {
            Some(HtmlType::Text)
        } else if input.peek(Token![::]) {
            Some(HtmlType::Component)
        } else if input.peek(Ident::peek_any) {
            let ident: Ident = input.parse().ok()?;
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
            None => return Err(input.error("unexpected token")),
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
        match self {
            Self::Component(component) => component.to_tokens(tokens),
            Self::Element(element) => element.to_tokens(tokens),
            Self::Text(text) => text.to_tokens(tokens),
        }
    }
}

/// A macro for ergonomically creating complex UI structures.
///
/// TODO: write some more docs
#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as element::Element);

    let quoted = quote! {
        ::maple_core::TemplateResult::new(#input)
    };

    TokenStream::from(quoted)
}

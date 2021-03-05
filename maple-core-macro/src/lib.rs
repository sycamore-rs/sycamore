mod children;
mod element;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Result};

pub(crate) enum HtmlTree {
    Element(element::HtmlElement),
}

impl Parse for HtmlTree {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self::Element(input.parse()?))
    }
}

impl ToTokens for HtmlTree {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            HtmlTree::Element(element) => element.to_tokens(tokens),
        }
    }
}

#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as element::HtmlElement);

    TokenStream::from(input.to_token_stream())
}

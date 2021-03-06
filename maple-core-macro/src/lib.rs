mod children;
mod element;
mod text;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Result, Token};

pub(crate) enum HtmlTree {
    Element(element::HtmlElement),
    Text(text::Text),
}

impl Parse for HtmlTree {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![#]) {
            Ok(Self::Text(input.parse()?))
        } else {
            Ok(Self::Element(input.parse()?))
        }
    }
}

impl ToTokens for HtmlTree {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Element(element) => element.to_tokens(tokens),
            Self::Text(text) => text.to_tokens(tokens),
        }
    }
}

#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as element::HtmlElement);

    let quoted = quote! {
        || { #input }
    };

    TokenStream::from(quoted)
}

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::token::Paren;
use syn::{parenthesized, Expr, LitStr, Result};

pub enum Text {
    Text(LitStr),
    Splice(Paren, Box<Expr>),
}

impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Paren) {
            let content;
            let paren = parenthesized!(content in input);
            Ok(Self::Splice(paren, content.parse()?))
        } else {
            Ok(Self::Text(input.parse()?))
        }
    }
}

impl ToTokens for Text {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Text::Text(text) => {
                let quoted = text.to_token_stream();
                tokens.extend(quoted);
            }
            Text::Splice(_, expr) => {
                let quoted = expr.to_token_stream();
                tokens.extend(quoted);
            }
        }
    }
}

use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{braced, token, Ident, Result, Token};

use crate::HtmlTree;

pub(crate) struct Children {
    pub brace_token: token::Brace,
    pub body: Vec<HtmlTree>,
}

impl Parse for Children {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);
        let mut body = Vec::new();

        while content.peek(Ident::peek_any) || content.peek(Token![#]) || content.peek(Token![@]) {
            body.push(content.parse()?);
        }

        Ok(Self {
            brace_token,
            body,
        })
    }
}

use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, Result, Token};

pub(crate) struct Text {
    _hash_token: Token![#],
    expr: Expr,
}

impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            _hash_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Text {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Text { _hash_token, expr } = self;

        let expr_span = expr.span();
        let quoted = quote_spanned! {expr_span=>
            ::maple_core::internal::text(&::std::format!("{}", #expr))
        };
        tokens.extend(quoted);
    }
}

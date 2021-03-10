use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, Result, Token};

use crate::children::Children;

pub(crate) struct IfElse {
    pub at_token: Token![@],
    pub if_token: Token![if],
    pub if_condition: Expr,
    pub children: Children,
}

impl Parse for IfElse {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            at_token: input.parse()?,
            if_token: input.parse()?,
            if_condition: input.call(Expr::parse_without_eager_brace)?,
            children: input.parse()?,
        })
    }
}

impl ToTokens for IfElse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let IfElse {
            at_token: _,
            if_token: _,
            if_condition,
            children,
        } = self;

        let if_condition = quote_spanned! { if_condition.span()=>
            #if_condition
        };

        let mut append_children = Vec::new();
        for child in &children.body {
            append_children.push(quote! {
                ::maple_core::internal::append(&element, &&#child);
            });
        }

        let quoted = quote! {
            if #if_condition {
                let element = ::maple_core::internal::fragment();
                #(#append_children)*
                element
            } else {
                ::maple_core::internal::fragment()
            }
        };

        tokens.extend(quoted);
    }
}

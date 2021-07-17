use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::token::Paren;
use syn::{parenthesized, Expr, Result};

pub struct Splice {
    pub paren: Paren,
    pub expr: Expr,
}

impl Parse for Splice {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren = parenthesized!(content in input);
        Ok(Self {
            paren,
            expr: content.parse()?,
        })
    }
}

impl ToTokens for Splice {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // match self {
        //     Text::Str(text) => {
        //         // Since this is static text, intern it as it will likely be constructed many
        // times.         let quoted = quote! {
        //             if ::std::cfg!(target_arch = "wasm32") {
        //                 ::sycamore::rt::intern(#text)
        //             } else {
        //                 #text
        //             }
        //         };
        //         tokens.extend(quoted);
        //     }
        //     Text::Splice(_, expr) => {
        let Self { paren: _, expr } = self;
        let quoted = expr.to_token_stream();
        tokens.extend(quoted);
        // }
        // }
    }
}

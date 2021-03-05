use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, LitStr};

pub struct HtmlTag {
    tag: Ident,
}

impl Parse for HtmlTag {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(HtmlTag {
            tag: input.call(Ident::parse_any)?,
        })
    }
}

impl ToTokens for HtmlTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let HtmlTag { tag } = self;

        let tag_str = LitStr::new(&tag.to_string(), tag.span());

        let quoted = quote! {
            ::maple_core::internal::element(#tag_str)
        };

        tokens.extend(quoted);
    }
}

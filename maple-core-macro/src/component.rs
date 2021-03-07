use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{ExprCall, Result};

/// Components are identical to function calls.
pub(crate) struct Component {
    expr_call: ExprCall,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            expr_call: input.parse()?,
        })
    }
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Component { expr_call } = self;

        let quoted = quote! { #expr_call };

        tokens.extend(quoted);
    }
}

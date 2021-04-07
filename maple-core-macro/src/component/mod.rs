use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::Result;

pub struct ComponentFunctionName {}

impl Parse for ComponentFunctionName {
    fn parse(input: ParseStream) -> Result<Self> {
        let _ = input;
        todo!()
    }
}

impl ToTokens for ComponentFunctionName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let quoted = quote! {};

        tokens.extend(quoted);
    }
}

pub struct ComponentFunction {}

impl Parse for ComponentFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let _ = input;
        todo!()
    }
}

impl ToTokens for ComponentFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let quoted = quote! {};

        tokens.extend(quoted);
    }
}

pub fn impl_component(
    attr: ComponentFunctionName,
    component: ComponentFunction,
) -> proc_macro::TokenStream {
    let _ = attr;
    component.to_token_stream().into()
}

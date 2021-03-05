mod element;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as element::HtmlElement);

    TokenStream::from(input.to_token_stream())
}

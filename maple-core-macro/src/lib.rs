use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

mod template;

/// A macro for ergonomically creating complex UI structures.
///
/// TODO: write some more docs
#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as template::HtmlRoot);

    TokenStream::from(input.to_token_stream())
}

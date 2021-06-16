use proc_macro::TokenStream;
use syn::parse_macro_input;

/// The `Router` procedural macro.
#[proc_macro_derive(Router)]
pub fn router(routes: TokenStream) -> TokenStream {
    todo!()
}

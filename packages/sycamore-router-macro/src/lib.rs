mod parser;
mod router;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// The `Router` procedural macro.
#[proc_macro_derive(Router, attributes(to, not_found))]
pub fn router(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    router::router_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

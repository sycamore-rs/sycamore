mod parser;
mod route;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// The `Route` procedural macro.
///
/// This macro derives the `Route` trait for the given `enum`.
#[proc_macro_derive(Route, attributes(to, not_found, preload))]
pub fn route(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    route::route_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

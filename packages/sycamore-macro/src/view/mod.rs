//! The `view!` macro implementation.

#![allow(clippy::mixed_read_write_in_expression)] // Needed when using `syn::parenthesized!`.

pub mod codegen;
pub mod ir;
pub mod parse;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Expr, Result, Token};

use self::codegen::Codegen;
use self::ir::*;

pub struct WithcxArg<T> {
    cx: Expr,
    rest: T,
}

impl<T: Parse> Parse for WithcxArg<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let cx = input.parse()?;
        let _comma: Token![,] = input.parse().map_err(|_| input.error("expected `,` (help: make sure you pass the cx variable to the macro as an argument)"))?;
        let rest = input.parse()?;
        Ok(Self { cx, rest })
    }
}

pub fn view_impl(view_root: WithcxArg<ViewRoot>) -> TokenStream {
    let cx = view_root.cx;
    let codegen_state = Codegen {
        cx: parse_quote!(#cx),
    };
    let quoted = codegen_state.view_root(&view_root.rest);
    quote! {{
        #[allow(unused_variables)]
        let #cx: ::sycamore::reactive::BoundedScope = #cx; // Make sure that cx is used.
        #quoted
    }}
}

pub fn node_impl(elem: WithcxArg<Element>) -> TokenStream {
    let cx = elem.cx;
    let codegen_state = Codegen {
        cx: parse_quote!(#cx),
    };
    let quoted = codegen_state.element(&elem.rest);
    quote! {{
        #[allow(unused_variables)]
        let #cx: ::sycamore::reactive::BoundedScope = #cx; // Make sure that cx is used.
        #quoted
    }}
}

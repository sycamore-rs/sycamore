//! The `view!` macro implementation.

#![allow(clippy::eval_order_dependence)] // Needed when using `syn::parenthesized!`.

pub mod codegen;
pub mod ir;
pub mod parse;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Expr, Result, Token};

use self::codegen::Codegen;
use self::ir::*;

pub struct WithCtxArg<T> {
    ctx: Expr,
    rest: T,
}

impl<T: Parse> Parse for WithCtxArg<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let ctx = input.parse()?;
        let _comma: Token![,] = input.parse().map_err(|_| input.error("expected `,` (help: make sure you pass the ctx variable to the macro as an argument)"))?;
        let rest = input.parse()?;
        Ok(Self { ctx, rest })
    }
}

pub fn view_impl(view_root: WithCtxArg<ViewRoot>) -> TokenStream {
    let ctx = view_root.ctx;
    let codegen_state = Codegen {
        ctx: parse_quote!(#ctx),
    };
    let quoted = codegen_state.view_root(&view_root.rest);
    quote! {{
        #[allow(unused_variables)]
        let #ctx: ::sycamore::reactive::ScopeRef = &#ctx; // Make sure that ctx is used.
        #quoted
    }}
}

pub fn node_impl(elem: WithCtxArg<Element>) -> TokenStream {
    let ctx = elem.ctx;
    let codegen_state = Codegen {
        ctx: parse_quote!(#ctx),
    };
    let quoted = codegen_state.element(&elem.rest);
    quote! {{
        #[allow(unused_variables)]
        let #ctx: ::sycamore::reactive::ScopeRef = &#ctx; // Make sure that ctx is used.
        #quoted
    }}
}

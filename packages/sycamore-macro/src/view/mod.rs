//! The `view!` macro implementation.

#![allow(clippy::mixed_read_write_in_expression)] // Needed when using `syn::parenthesized!`.

pub mod codegen;
pub mod ir;
pub mod parse;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Expr, Path, Result, Token};

use self::codegen::Codegen;
use self::ir::*;

pub struct WithArgs<T> {
    elements_mod_path: Path,
    cx: Expr,
    rest: T,
}

impl<T: Parse> Parse for WithArgs<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let elements_mod_path = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let cx = input.parse()?;
        let _comma: Token![,] = input.parse().map_err(|_| input.error("expected `,` (help: make sure you pass the cx variable to the macro as an argument)"))?;
        let rest = input.parse()?;
        Ok(Self {
            elements_mod_path,
            cx,
            rest,
        })
    }
}

pub fn view_impl(view_root: WithArgs<ViewRoot>) -> TokenStream {
    let WithArgs {
        elements_mod_path,
        cx,
        rest: view_root,
    } = view_root;
    let codegen_state = Codegen {
        elements_mod_path,
        cx: parse_quote!(#cx),
    };
    let quoted = codegen_state.view_root(&view_root);
    quote! {{
        #[allow(unused_variables)]
        let #cx: ::sycamore::reactive::BoundedScope = #cx; // Make sure that cx is used.
        #quoted
    }}
}

pub fn node_impl(_elem: WithArgs<Element>) -> TokenStream {
    // let WithArgs {
    //     elements_mod_path,
    //     cx,
    //     rest: elem,
    // } = elem;
    // let codegen_state = Codegen {
    //     elements_mod_path,
    //     cx: parse_quote!(#cx),
    // };
    // let quoted = codegen_state.element(&elem);
    // quote! {{
    //     #[allow(unused_variables)]
    //     let #cx: ::sycamore::reactive::BoundedScope = #cx; // Make sure that cx is used.
    //     #quoted
    // }}
    todo!();
}

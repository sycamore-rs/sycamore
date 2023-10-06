//! The `view!` macro implementation.

#![allow(clippy::mixed_read_write_in_expression)] // Needed when using `syn::parenthesized!`.

pub mod codegen;
pub mod ir;
pub mod parse;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Path, Result, Token};

use self::codegen::Codegen;
use self::ir::*;

pub struct WithArgs<T> {
    elements_mod_path: Path,
    rest: T,
}

impl<T: Parse> Parse for WithArgs<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let elements_mod_path = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let rest = input.parse()?;
        Ok(Self {
            elements_mod_path,
            rest,
        })
    }
}

pub fn view_impl(view_root: WithArgs<ViewRoot>) -> TokenStream {
    let WithArgs {
        elements_mod_path,
        rest: view_root,
    } = view_root;
    let codegen_state = Codegen { elements_mod_path };
    codegen_state.view_root(&view_root)
}

pub fn node_impl(elem: WithArgs<Element>) -> TokenStream {
    let WithArgs {
        elements_mod_path,
        rest: elem,
    } = elem;
    let codegen_state = Codegen { elements_mod_path };
    let quoted = codegen_state.view_node(&ViewNode::Element(elem));
    quote! {{
        let __view = #quoted;
        ::std::clone::Clone::clone(__view.as_node().unwrap())
    }}
}

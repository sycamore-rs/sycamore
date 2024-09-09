//! Proc-macros used in [Sycamore](https://sycamore-rs.netlify.app).

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod component;
mod props;
mod view;

/// A macro for ergonomically creating complex UI complex layouts.
///
/// To learn more about the view syntax, see [the chapter on views](https://sycamore-rs.netlify.app/docs/basics/view)
/// in the Sycamore Book.
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as sycamore_view_parser::ir::Root);

    view::Codegen {}.root(&root).into()
}

/// A macro for creating components from functions.
///
/// Add this attribute to a `fn` to create a component from that function.
///
/// To learn more about components, see the chapter on
/// [components](https://sycamore-rs.netlify.app/docs/basics/components) in the Sycamore Book.
#[proc_macro_attribute]
pub fn component(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as component::ComponentArgs);

    component::component_impl(args, item.clone().into())
        .unwrap_or_else(|err| {
            // If proc-macro errors, emit the original function for better IDE support.
            let error_tokens = err.into_compile_error();
            let body_input = proc_macro2::TokenStream::from(item);
            quote! {
                #body_input
                #error_tokens
            }
        })
        .into()
}

/// The derive macro for `Props`. The macro creates a builder-like API used in the [`view!`] macro.
#[proc_macro_derive(Props, attributes(prop))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    props::impl_derive_props(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// A macro for feature gating code that should only be run on the server.
///
/// By default, the target is used to determine the rendering mode. However, `--cfg
/// sycamore_force_ssr` can be used to override this behavior.
///
/// # Example
///
/// ```
/// # use sycamore_macro::*;
/// #[cfg_ssr]
/// fn server_only() {}
/// ```
///
/// See also [`macro@cfg_not_ssr`].
#[proc_macro_attribute]
pub fn cfg_ssr(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    quote! {
        #[cfg(any(not(target_arch = "wasm32"), sycamore_force_ssr))]
        #input
    }
    .into()
}

/// A macro for feature gating code that should only be run on the web.
///
/// By default, the target is used to determine the rendering mode. However, `--cfg
/// sycamore_force_ssr` can be used to override this behavior.
///
/// # Example
///
/// ```
/// # use sycamore_macro::*;
/// #[cfg_not_ssr]
/// fn browser_only() {}
/// ```
///
/// See also [`macro@cfg_ssr`].
#[proc_macro_attribute]
pub fn cfg_not_ssr(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    quote! {
        #[cfg(all(target_arch = "wasm32", not(sycamore_force_ssr)))]
        #input
    }
    .into()
}

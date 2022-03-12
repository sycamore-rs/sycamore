//! Proc-macros used in [Sycamore](https://sycamore-rs.netlify.app).

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod component;
mod prop;
mod view;

/// A macro for ergonomically creating complex UI structures.
///
/// To learn more about the template syntax, see the chapter on
/// [the `view!` macro](https://sycamore-rs.netlify.app/docs/basics/view) in the Sycamore Book.
#[proc_macro]
pub fn view(view: TokenStream) -> TokenStream {
    let view_root = parse_macro_input!(view as view::WithCtxArg<view::ir::ViewRoot>);

    view::view_impl(view_root).into()
}

/// ```
/// use sycamore::prelude::*;
///
/// #[component]
/// pub fn MyComponent<G: Html>(ctx: Scope) -> View<G> {
///     let cool_button: G = node! { ctx, button { "The coolest 😎" } };
///
///     cool_button.set_property("myProperty", &"Epic!".into());
///
///     View::new_node(cool_button)
/// }
/// ```
#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    let elem = parse_macro_input!(input as view::WithCtxArg<view::ir::Element>);

    view::node_impl(elem).into()
}

/// A macro for creating components from functions.
///
/// Add this attribute to a `fn` to create a component from that function.
///
/// To learn more about components, see the chapter on
/// [components](https://sycamore-rs.netlify.app/docs/basics/components) in the Sycamore Book.
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, component: TokenStream) -> TokenStream {
    let comp = {
        let component = component.clone();
        parse_macro_input!(component as component::ComponentFunction)
    };

    component::component_impl(comp)
        .unwrap_or_else(|err| {
            // If proc-macro errors, emit the original function for better IDE support.
            let error_tokens = err.into_compile_error();
            let component_tokens = proc_macro2::TokenStream::from(component);
            quote! {
                #component_tokens
                #error_tokens
            }
        })
        .into()
}

/// A derive macro for creating a builder-like API used in the [`view!`] macro.
#[proc_macro_derive(Prop, attributes(builder))]
pub fn derive_prop(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    prop::impl_derive_prop(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

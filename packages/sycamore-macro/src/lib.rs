use proc_macro::TokenStream;
use syn::parse_macro_input;

mod component;
mod view;

/// A macro for ergonomically creating complex UI structures.
///
/// To learn more about the template syntax, see the chapter on
/// [the `view!` macro](https://sycamore-rs.netlify.app/docs/basics/view) in the Sycamore Book.
#[proc_macro]
pub fn view(component: TokenStream) -> TokenStream {
    let component = parse_macro_input!(component as view::HtmlRoot);

    view::view_impl(component).into()
}

/// ```
/// use sycamore::prelude::*;
///
/// #[component(MyComponent<G>)]
/// pub fn my_component() -> View<G> {
///     let cool_button: G = node! { button { "The coolest 😎" } };
///
///     cool_button.set_property("myProperty", &"Epic!".into());
///
///     View::new_node(cool_button)
/// }
/// ```
#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    let node = parse_macro_input!(input as view::Element);

    view::node_impl(node).into()
}

/// A macro for creating components from functions.
///
/// Add this attribute to a `fn` to create a component from that function.
///
/// To learn more about components, see the chapter on
/// [components](https://sycamore-rs.netlify.app/docs/basics/components) in the Sycamore Book.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, component: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as component::ComponentFunctionName);
    let component = parse_macro_input!(component as component::ComponentFunction);

    component::component_impl(attr, component)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

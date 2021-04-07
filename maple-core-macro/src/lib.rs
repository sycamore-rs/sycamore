use proc_macro::TokenStream;
use syn::parse_macro_input;

mod component;
mod template;

/// A macro for ergonomically creating complex UI structures.
///
/// TODO: write some more docs
#[proc_macro]
pub fn template(component: TokenStream) -> TokenStream {
    let component = parse_macro_input!(component as template::HtmlRoot);

    template::template_impl(component)
}

/// A macro for creating components from functions.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, component: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as component::ComponentFunctionName);
    let component = parse_macro_input!(component as component::ComponentFunction);

    component::impl_component(attr, component)
}

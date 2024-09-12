//! Implementation of rendering backend.

use crate::*;

cfg_not_ssr_item!(
    mod dom_node;
);
cfg_not_ssr_item!(
    #[cfg(feature = "hydrate")]
    mod hydrate_node;
);
cfg_ssr_item!(
    mod ssr_node;
);
mod dom_render;
mod ssr_render;

// We add this so that we get IDE support in Rust Analyzer.
#[cfg(rust_analyzer)]
mod dom_node;
#[cfg(rust_analyzer)]
mod hydrate_node;

#[cfg_not_ssr]
pub use dom_node::*;
pub use dom_render::*;
#[cfg_not_ssr]
#[cfg(feature = "hydrate")]
pub use hydrate_node::*;
#[cfg_ssr]
pub use ssr_node::*;
pub use ssr_render::*;

/// A trait that should be implemented for anything that represents an HTML node.
pub trait ViewHtmlNode: ViewNode {
    /// Create a new HTML element.
    fn create_element(tag: Cow<'static, str>) -> Self;
    /// Create a new HTML element with a XML namespace.
    fn create_element_ns(namespace: &'static str, tag: Cow<'static, str>) -> Self;
    /// Create a new HTML text node.
    fn create_text_node(text: Cow<'static, str>) -> Self;
    /// Create a new HTML text node whose value will be changed dynamically.
    fn create_dynamic_text_node(text: Cow<'static, str>) -> Self {
        Self::create_text_node(text)
    }
    /// Create a new HTML marker (comment) node.
    fn create_marker_node() -> Self;

    /// Set an HTML attribute.
    fn set_attribute(&mut self, name: Cow<'static, str>, value: MaybeDynString);
    /// Set a boolean HTML attribute.
    fn set_bool_attribute(&mut self, name: Cow<'static, str>, value: MaybeDynBool);
    /// Set a JS property on an element.
    fn set_property(&mut self, name: Cow<'static, str>, value: MaybeDynJsValue);
    /// Set an event handler on an element.
    fn set_event_handler(
        &mut self,
        name: Cow<'static, str>,
        handler: impl FnMut(web_sys::Event) + 'static,
    );
    /// Set the inner HTML value of an element.
    fn set_inner_html(&mut self, inner_html: Cow<'static, str>);

    /// Return the raw web-sys node.
    fn as_web_sys(&self) -> &web_sys::Node;
    /// Wrap a raw web-sys node.
    fn from_web_sys(node: web_sys::Node) -> Self;
}

/// A trait for unwrapping a type into an `HtmlNode`.
pub trait IntoHtmlNode {
    fn into_html_node(self) -> HtmlNode;
    fn as_html_node(&self) -> &HtmlNode;
    fn as_html_node_mut(&mut self) -> &mut HtmlNode;
}

/// A trait that represents an attribute that can be set. This is not "attribute" in the HTML spec
/// sense. It can also represent JS properties (and possibly more ...) that can be set on an HTML
/// element.
pub trait AttributeValue {
    fn set_self(self, el: &mut HtmlNode, name: &'static str);
}

impl AttributeValue for MaybeDynString {
    fn set_self(self, el: &mut HtmlNode, name: &'static str) {
        el.set_attribute(name.into(), self);
    }
}

impl AttributeValue for MaybeDynBool {
    fn set_self(self, el: &mut HtmlNode, name: &'static str) {
        el.set_bool_attribute(name.into(), self);
    }
}

impl AttributeValue for MaybeDynJsValue {
    fn set_self(self, el: &mut HtmlNode, name: &'static str) {
        el.set_property(name.into(), self);
    }
}

thread_local! {
    /// Whether we are in hydration mode or not.
    pub(crate) static IS_HYDRATING: Cell<bool> = const { Cell::new(false) };
}

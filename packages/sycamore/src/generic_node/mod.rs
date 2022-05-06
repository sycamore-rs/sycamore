//! Abstraction over a rendering backend.

#[cfg(feature = "dom")]
pub mod dom_node;
#[cfg(all(feature = "dom", feature = "hydrate"))]
pub mod hydrate_dom;
#[cfg(feature = "ssr")]
pub mod ssr_node;

#[cfg(feature = "dom")]
pub use dom_node::*;
#[cfg(all(feature = "dom", feature = "hydrate"))]
pub use hydrate_dom::*;
#[cfg(feature = "ssr")]
pub use ssr_node::*;
pub use sycamore_core::generic_node::*;
use wasm_bindgen::JsValue;
use web_sys::Event;

/// Trait that is implemented by all [`GenericNode`] backends that render to HTML.
pub trait Html: GenericNode<EventType = Event, PropertyType = JsValue> {
    /// A boolean indicating whether this node is rendered in a browser context.
    ///
    /// A value of `false` does not necessarily mean that it is not being rendered in WASM or even
    /// in the browser. It only means that it does not create DOM nodes.
    const IS_BROWSER: bool;
}

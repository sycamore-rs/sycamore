//! Abstraction over a rendering backend.

#[cfg(feature = "dom")]
pub mod dom_node;
#[cfg(all(feature = "dom", feature = "experimental-hydrate"))]
pub mod hydrate_dom;
#[cfg(feature = "ssr")]
pub mod ssr_node;

use std::fmt;
use std::hash::Hash;

use wasm_bindgen::prelude::*;
use web_sys::Event;

use crate::reactive::ScopeRef;

#[cfg(feature = "dom")]
pub use dom_node::*;
#[cfg(all(feature = "dom", feature = "experimental-hydrate"))]
pub use hydrate_dom::*;
#[cfg(feature = "ssr")]
pub use ssr_node::*;

/// Abstraction over a rendering backend.
///
/// You would probably use this trait as a trait bound when you want to accept any rendering
/// backend. For example, components are often generic over [`GenericNode`] to be able to render to
/// different backends.
///
/// Note that components are **NOT** represented by [`GenericNode`]. Instead, components are
/// _disappearing_, meaning that they are simply functions that generate [`GenericNode`]s inside a
/// new reactive context. This means that using components add minimal overhead.
///
/// Sycamore ships with 2 rendering backends out of the box:
/// * [`DomNode`] - Rendering in the browser (to real DOM nodes).
/// * [`SsrNode`] - Render to a static string (often on the server side for Server Side Rendering,
///   aka. SSR).
///
/// To implement your own rendering backend, you will need to create a new struct which implements
/// [`GenericNode`].
///
/// # Cloning
///
/// [`GenericNode`]s should be cheaply cloneable (usually backed by a [`Rc`](std::rc::Rc) or other
/// reference counted container) and preserve reference equality.
pub trait GenericNode: fmt::Debug + Clone + PartialEq + Eq + Hash + 'static {
    /// The type of the event that is passed to the event handler.
    type EventType;

    /// Whether this rendering backend needs the hydration registry.
    const USE_HYDRATION_CONTEXT: bool = false;
    const CLIENT_SIDE_HYDRATION: bool = false;

    /// Create a new element node.
    fn element(tag: &str) -> Self;

    /// Create a new text node.
    fn text_node(text: &str) -> Self;

    /// Create a marker (dummy) node. For [`DomNode`], this is implemented by creating an empty
    /// comment node. This is used, for example, in [`Keyed`](crate::flow::Keyed) and
    /// [`Indexed`](crate::flow::Indexed) for scenarios where you want to push a new item to the
    /// end of the list. If the list is empty, a dummy node is needed to store the position of
    /// the component.
    fn marker() -> Self {
        Self::marker_with_text("")
    }

    /// Create a marker (dummy) node with text content. For empty marker, prefer
    /// [`GenericNode::marker`] instead.
    fn marker_with_text(text: &str) -> Self;

    /// Sets an attribute on a node.
    fn set_attribute(&self, name: &str, value: &str);

    /// Removes an attribute on a node.
    fn remove_attribute(&self, name: &str);

    /// Sets the `class` attribute on a node.
    /// This should have the same outcome as calling `set_attribute("class", value)`.
    /// For [`DomNode`], this sets the `className` property directly which is about 2x faster (on
    /// Chrome).
    fn set_class_name(&self, value: &str);

    fn add_class(&self, class: &str);

    fn remove_class(&self, class: &str);

    /// Sets a property on a node.
    fn set_property(&self, name: &str, value: &JsValue);

    /// Removes a property on a node.
    fn remove_property(&self, name: &str);

    /// Appends a child to the node's children.
    fn append_child(&self, child: &Self);

    /// Get the first child of the node.
    fn first_child(&self) -> Option<Self>;

    /// Insert a new child node to this node's children. If `reference_node` is `Some`, the child
    /// will be inserted before the reference node. Else if `None`, the child will be inserted
    /// at the end.
    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>);

    /// Remove a child node from this node's children.
    fn remove_child(&self, child: &Self);

    /// Replace a child node from this node's children with a new child node.
    fn replace_child(&self, old: &Self, new: &Self);

    /// Insert a new node before this node.
    fn insert_sibling_before(&self, child: &Self);

    /// Returns the parent node, or `None` if detached.
    fn parent_node(&self) -> Option<Self>;

    /// Returns the next sibling, or `None` if this node is the last sibling.
    fn next_sibling(&self) -> Option<Self>;

    /// Remove this node from the tree.
    fn remove_self(&self);

    /// Add a event handler to the event `name`.
    fn event<'a>(&self, ctx: ScopeRef<'a>, name: &str, handler: Box<dyn Fn(Self::EventType) + 'a>);

    /// Update inner text of the node. If the node has elements, all the elements are replaced with
    /// a new text node.
    fn update_inner_text(&self, text: &str);

    /// Updates the inner html of the node.
    /// The html will not be parsed in non-browser environments. This means that accessing methods
    /// such as [`first_child`](GenericNode::first_child) will return `None`.
    fn dangerously_set_inner_html(&self, html: &str);

    /// Create a deep clone of the node.
    #[must_use = "clone_node returns a new node"]
    fn clone_node(&self) -> Self;
}

/// Trait that is implemented by all [`GenericNode`] backends that render to HTML.
pub trait Html: GenericNode<EventType = Event> {
    /// A boolean indicating whether this node is rendered in a browser context.
    ///
    /// A value of `false` does not necessarily mean that it is not being rendered in WASM or even
    /// in the browser. It only means that it does not create DOM nodes.
    const IS_BROWSER: bool;
}

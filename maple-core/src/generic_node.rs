//! Abstraction over a rendering backend.

#[cfg(feature = "dom")]
pub mod dom_node;
#[cfg(feature = "ssr")]
pub mod ssr_node;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::Event;

use crate::prelude::*;
#[cfg(feature = "dom")]
pub use dom_node::*;
#[cfg(feature = "ssr")]
pub use ssr_node::*;

pub type EventListener = dyn Fn(Event);

/// Abstraction over a rendering backend.
///
/// You would probably use this trait as a trait bound when you want to accept any rendering
/// backend. For example, components are often generic over [`GenericNode`] to be able to render to
/// different backends.
///
/// Note that components are **NOT** represented by [`GenericNode`]. Instead, components are
/// _disappearing_, meaning that they are simply functions that generate [`GenericNode`]s inside a
/// new reactive context. This means that there is no overhead whatsoever when using components.
///
/// Maple ships with 2 rendering backends out of the box:
/// * [`DomNode`] - Rendering in the browser (to real DOM nodes).
/// * [`SsrNode`] - Render to a static string (often on the server side for Server Side Rendering,
///   aka. SSR).
///
/// To implement your own rendering backend, you will need to create a new struct which implements
/// [`GenericNode`].
pub trait GenericNode: fmt::Debug + Clone + PartialEq + Eq + 'static {
    /// Create a new element node.
    fn element(tag: &str) -> Self;

    /// Create a new text node.
    fn text_node(text: &str) -> Self;

    /// Create a new fragment (list of nodes). A fragment is not necessarily wrapped around by an
    /// element.
    fn fragment() -> Self;

    /// Create a marker (dummy) node. For [`DomNode`], this is implemented by creating an empty
    /// comment node. This is used, for example, in [`Keyed`] and [`Indexed`] for scenarios
    /// where you want to push a new item to the end of the list. If the list is empty, a dummy
    /// node is needed to store the position of the component.
    fn marker() -> Self;

    /// Sets an attribute on a node.
    fn set_attribute(&self, name: &str, value: &str);

    /// Sets a property on a node.
    fn set_property(&self, name: &str, value: &JsValue);

    /// Appends a child to the node's children.
    fn append_child(&self, child: &Self);

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
    ///
    /// TODO: Remove this node on Drop.
    fn remove_self(&self);

    /// Add a [`EventListener`] to the event `name`.
    fn event(&self, name: &str, handler: Box<EventListener>);

    /// Update inner text of the node. If the node has elements, all the elements are replaced with
    /// a new text node.
    fn update_inner_text(&self, text: &str);

    /// Append an item that implements [`Render`] and automatically updates the DOM inside an
    /// effect.
    fn append_render(&self, child: Box<dyn Fn() -> Box<dyn Render<Self>>>) {
        let parent = self.clone();

        let nodes = create_effect_initial(cloned!((parent) => move || {
            let node = RefCell::new(child().create());

            let effect = cloned!((node) => move || {
                let new_node = child().update_node(&parent, &node.borrow());
                *node.borrow_mut() = new_node;
            });

            (Rc::new(effect), node)
        }));

        for node in nodes.borrow().iter() {
            parent.append_child(node);
        }
    }
}

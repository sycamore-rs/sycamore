//! Generic rendering backend.

use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;

use sycamore_reactive::Scope;

use crate::view::View;

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
/// Sycamore ships with a few rendering backends out of the box. Here are some examples:
/// * `DomNode` - Rendering in the browser (to real DOM nodes).
/// * `SsrNode` - Render to a static string (often on the server side for Server Side Rendering,
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
    /// Create a new text node.
    fn text_node(cx: Scope, text: Cow<'static, str>) -> Self;

    /// Create a marker (dummy) node. For `DomNode`, this is implemented by creating an empty
    /// comment node. This is used, for example, in `Keyed` and `Indexed` for scenarios where you
    /// want to push a new item to the end of the list. If the list is empty, a dummy node is
    /// needed to store the position of the component.
    fn marker(cx: Scope) -> Self {
        Self::marker_with_text(cx, "".into())
    }

    /// Create a marker (dummy) node with text content. For empty marker, prefer
    /// [`GenericNode::marker`] instead.
    fn marker_with_text(cx: Scope, text: Cow<'static, str>) -> Self;

    /// Sets an attribute on a node.
    /// If the attribute does not exist, it is added. If the attribute already exists, the value is
    /// overridden.
    fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>);

    /// Removes an attribute on a node.
    fn remove_attribute(&self, name: Cow<'static, str>);

    /// Sets the `class` attribute on a node.
    /// This should have the same outcome as calling `set_attribute("class", value)`.
    /// For `DomNode`, this sets the `className` property directly which is about 2x faster (on
    /// Chrome).
    fn set_class_name(&self, value: Cow<'static, str>);

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

    /// Update inner text of the node. If the node has elements, all the elements are replaced with
    /// a new text node.
    fn update_inner_text(&self, text: Cow<'static, str>);

    /// Updates the inner html of the node.
    /// The html will not be parsed in non-browser environments. This means that accessing methods
    /// such as [`first_child`](GenericNode::first_child) will return `None`.
    fn dangerously_set_inner_html(&self, html: Cow<'static, str>);

    /// Create a deep clone of the node.
    #[must_use = "clone_node returns a new node"]
    fn clone_node(&self) -> Self;

    /// A callback that is called after a element is finished building with
    /// [`ElementBuilder::finish`](crate::elements::ElementBuilder::finish).
    fn finish_element(&mut self, _cx: Scope, _is_dyn: bool) {}

    /// Get the next element when building the element tree using
    /// [`ElementBuilder`](crate::elements::ElementBuilder).
    ///
    /// By default, this returns `None` which means that a new element will be created from scratch.
    fn get_next_element(_cx: Scope, f: impl Fn() -> Self) -> Self {
        f()
    }

    /// A wrapper around [`insert`](crate::render::insert) that allows you to customize the behavior
    /// for the rendering backend.
    fn builder_insert(&self, cx: Scope, view: View<Self>) {
        crate::render::insert(cx, self, view, None, None, true);
    }

    /// A wrapper method to customize the behavior when functions are instantiated.
    fn component_scope(_cx: Scope, f: impl FnOnce() -> View<Self>) -> View<Self> {
        f()
    }
}

/// Extension trait for [`GenericNode`] to provide additional methods related to element creation.
/// Not all backends need to implement this, but that means that you can only use components, not
/// elements in `view!`.
pub trait GenericNodeElements: GenericNode {
    type AnyEventData;

    /// Create a new element node from a tag string.
    fn element_from_tag(cx: Scope, tag: Cow<'static, str>) -> Self;

    /// Create a new namespaced element node from a tag string and a namespace string.
    ///
    /// The default implementation simply throws away the namespace and creates a normal element.
    /// This behavior can be overridden by backends that support namespaces.
    fn element_from_tag_namespace(
        cx: Scope,
        tag: Cow<'static, str>,
        _namespace: Cow<'static, str>,
    ) -> Self {
        Self::element_from_tag(cx, tag)
    }

    /// Attach a (type-erased) event listener to this element.
    fn add_event_listener<'a>(
        &self,
        cx: Scope<'a>,
        name: &str,
        listener: Box<dyn FnMut(Self::AnyEventData) + 'a>,
    );
}

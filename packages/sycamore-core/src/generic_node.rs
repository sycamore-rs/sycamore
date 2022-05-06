//! Generic rendering backend.

use std::fmt;
use std::hash::Hash;

use sycamore_reactive::Scope;

/// Represents an element.
pub trait SycamoreElement {
    /// The tag name of the element.
    const TAG_NAME: &'static str;
    /// The namespace of the element, or `None`, e.g. in the case of standard HTML5 elements.
    const NAME_SPACE: Option<&'static str>;
}

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
    /// The type of the event that is passed to the event handler.
    type EventType;
    /// The type for [`set_property`](Self::set_property).
    type PropertyType;

    /// Whether this rendering backend needs the hydration registry.
    const USE_HYDRATION_CONTEXT: bool = false;
    /// Whether this rendering backend hydrates nodes on the client side.
    const CLIENT_SIDE_HYDRATION: bool = false;

    /// Create a new element node.
    fn element<T: SycamoreElement>() -> Self;

    /// Create a new element node from a tag string.
    fn element_from_tag(tag: &str) -> Self;

    /// Create a new text node.
    fn text_node(text: &str) -> Self;

    /// Create a new text node from an integer.
    fn text_node_int(int: i32) -> Self {
        Self::text_node(&int.to_string())
    }

    /// Create a marker (dummy) node. For `DomNode`, this is implemented by creating an empty
    /// comment node. This is used, for example, in `Keyed` and `Indexed` for scenarios where you
    /// want to push a new item to the end of the list. If the list is empty, a dummy node is
    /// needed to store the position of the component.
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
    /// For `DomNode`, this sets the `className` property directly which is about 2x faster (on
    /// Chrome).
    fn set_class_name(&self, value: &str);

    /// Add a class to the element.
    /// If multiple classes are specified, delimited by spaces, all the classes should be added.
    /// Any classes that are already present should not be added a second time.
    fn add_class(&self, class: &str);

    /// Remove a class from the element.
    fn remove_class(&self, class: &str);

    /// Sets a property on a node.
    fn set_property(&self, name: &str, value: &Self::PropertyType);

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
    fn event<'a, F: FnMut(Self::EventType) + 'a>(&self, cx: Scope<'a>, name: &str, handler: F);

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

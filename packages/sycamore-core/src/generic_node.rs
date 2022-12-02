//! Generic rendering backend.

use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;

use sycamore_reactive::Scope;

use crate::render::insert;
use crate::view::View;

/// Represents an element (i.e. `div`, `p`, etc...).
pub trait SycamoreElement {
    /// The tag name of the element.
    const TAG_NAME: &'static str;
    /// The namespace of the element, or `None`, e.g. in the case of standard HTML5 elements.
    const NAMESPACE: Option<&'static str>;
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
    /// The type of the event that is passed to the event handler.
    type EventType;
    /// The type for [`set_property`](Self::set_property).
    type PropertyType;

    /// Whether this rendering backend needs the hydration registry.
    const USE_HYDRATION_CONTEXT: bool = false;
    /// Whether this rendering backend hydrates nodes on the client side.
    const CLIENT_SIDE_HYDRATION: bool = false;

    /// Create a new text node.
    fn text_node(text: Cow<'static, str>) -> Self;

    /// Create a new text node from an integer.
    fn text_node_int(int: i32) -> Self {
        Self::text_node(int.to_string().into())
    }

    /// Create a marker (dummy) node. For `DomNode`, this is implemented by creating an empty
    /// comment node. This is used, for example, in `Keyed` and `Indexed` for scenarios where you
    /// want to push a new item to the end of the list. If the list is empty, a dummy node is
    /// needed to store the position of the component.
    fn marker() -> Self {
        Self::marker_with_text("".into())
    }

    /// Create a marker (dummy) node with text content. For empty marker, prefer
    /// [`GenericNode::marker`] instead.
    fn marker_with_text(text: Cow<'static, str>) -> Self;

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
    /// The event should be removed once the scope is disposed, as to prevent accessing scope
    /// variables after the scope is disposed.
    fn event<'a, F: FnMut(Self::EventType) + 'a>(&self, cx: Scope<'a>, name: &str, handler: F);

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
}

/// Extension trait for [`GenericNode`] to provide additional methods related to element creation.
/// Not all backends need to implement this, but that means that you can only use components, not
/// elements in `view!`.
pub trait GenericNodeElements: GenericNode {
    /// Create a new element node.
    fn element<T: SycamoreElement>() -> Self;

    /// Create a new element node from a tag string.
    fn element_from_tag(tag: Cow<'static, str>) -> Self;

    /// Create a new namespaced element node from a tag string and a namespace string.
    ///
    /// The default implementation simply throws away the namespace and creates a normal element.
    /// This behavior can be overridden by backends that support namespaces.
    fn element_from_tag_namespace(tag: Cow<'static, str>, _namespace: Cow<'static, str>) -> Self {
        Self::element_from_tag(tag)
    }

    /// Instantiate a template by creating nodes to match the template structure. Returns the root
    /// node along with a list of flagged nodes and dynamic markers.
    ///
    /// By default, this will call [`instantiate_template_universal`]. However, this method can be
    /// overridden for specific node types to provide a more optimized implementation. For example,
    /// `SsrNode` could render the template to a static HTML string and then cache the result for
    /// reduced allocations and string formatting.
    fn instantiate_template(template: &Template) -> TemplateResult<Self> {
        instantiate_template_universal(template, InstantiateUniversalOpts::default())
    }

    /// Insert the dynamic values into the template at the dynamic markers.
    fn apply_dyn_values_to_template(
        cx: Scope,
        dyn_markers: &Vec<DynMarkerResult<Self>>,
        dyn_values: Vec<View<Self>>,
    ) {
        if dyn_markers.len() != dyn_values.len() {
            panic!(
                "The number of dynamic markers ({}) does not match the number of dynamic values ({}).",
                dyn_markers.len(),
                dyn_values.len()
            );
        }
        for (m, value) in dyn_markers.iter().zip(dyn_values.into_iter()) {
            insert(
                cx,
                &m.parent,
                value,
                m.initial.clone(),
                m.before.as_ref(),
                m.multi,
            );
        }
    }
}

/// Internal method to allow the proc-macro to call `G::instantiate_template` without knowing what
/// `G` is.
#[doc(hidden)]
pub fn __instantiate_template<G: GenericNodeElements>(template: &Template) -> TemplateResult<G> {
    G::instantiate_template(template)
}

/// Internal method to allow the proc-macro to call `G::apply_dyn_values_to_template` without
/// knowing what `G` is.
#[doc(hidden)]
pub fn __apply_dyn_values_to_template<G: GenericNodeElements>(
    cx: Scope,
    dyn_markers: &Vec<DynMarkerResult<G>>,
    dyn_values: Vec<View<G>>,
) {
    G::apply_dyn_values_to_template(cx, dyn_markers, dyn_values)
}

/// The "shape" of the template, i.e. what the structure of the template looks like. This is
/// basically the view with holes where the dynamic parts are and flags so that these holes can be
/// filled in later.
#[derive(Debug)]
pub enum TemplateShape {
    Element {
        tag: &'static str,
        ns: Option<&'static str>,
        children: &'static [TemplateShape],
        attributes: &'static [(&'static str, Cow<'static, str>)],
        flag: bool,
    },
    Text(&'static str),
    /// A dynamic view "hole". This is where a dynamic view should be inserted.
    DynMarker,
}

/// An unique identifier for an instantiated template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateId(pub u32);

/// A template that can be instantiated.
#[derive(Debug)]
pub struct Template {
    pub id: TemplateId,
    pub shape: TemplateShape,
}

/// Result of a template instantiation.
#[derive(Debug)]
pub struct TemplateResult<G: GenericNode> {
    pub root: G,
    pub flagged_nodes: Vec<G>,
    pub dyn_markers: Vec<DynMarkerResult<G>>,
}

/// Info extracted from a dynamic marker in a template.
#[derive(Debug)]
pub struct DynMarkerResult<G: GenericNode> {
    pub parent: G,
    pub before: Option<G>,
    pub initial: Option<View<G>>,
    pub multi: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InstantiateUniversalOpts {
    pub start_marker: Option<&'static str>,
    pub end_marker: Option<&'static str>,
}

fn instantiate_element_universal_impl<G: GenericNodeElements>(
    template: &TemplateShape,
    flagged_nodes: &mut Vec<G>,
    dyn_markers: &mut Vec<DynMarkerResult<G>>,
    opts: InstantiateUniversalOpts,
) -> G {
    match template {
        TemplateShape::Element {
            tag: ident,
            ns,
            children,
            attributes,
            flag,
        } => {
            let node = if let Some(ns) = ns {
                G::element_from_tag_namespace((*ident).into(), (*ns).into())
            } else {
                G::element_from_tag((*ident).into())
            };
            let multi = children.len() != 1;
            if *flag {
                flagged_nodes.push(node.clone());
            }
            for (name, value) in *attributes {
                node.set_attribute((*name).into(), value.clone());
            }
            for child in *children {
                instantiate_template_universal_impl(
                    &node,
                    child,
                    flagged_nodes,
                    dyn_markers,
                    multi,
                    opts,
                );
            }
            node
        }
        _ => panic!("expected an element"),
    }
}

fn instantiate_template_universal_impl<G: GenericNodeElements>(
    parent: &G,
    template: &TemplateShape,
    flagged_nodes: &mut Vec<G>,
    dyn_markers: &mut Vec<DynMarkerResult<G>>,
    multi: bool,
    opts: InstantiateUniversalOpts,
) {
    match template {
        TemplateShape::Element { .. } => {
            let node =
                instantiate_element_universal_impl(template, flagged_nodes, dyn_markers, opts);
            parent.append_child(&node);
        }
        TemplateShape::Text(text) => {
            let node = G::text_node((*text).into());
            parent.append_child(&node);
        }
        TemplateShape::DynMarker => {
            if multi {
                let start = match opts.start_marker {
                    Some(text) => G::marker_with_text(text.into()),
                    None => G::marker(),
                };
                parent.append_child(&start);
                let end = match opts.end_marker {
                    Some(text) => G::marker_with_text(text.into()),
                    None => G::marker(),
                };
                parent.append_child(&end);
                dyn_markers.push(DynMarkerResult {
                    parent: parent.clone(),
                    before: Some(end),
                    initial: None,
                    multi,
                });
            } else {
                dyn_markers.push(DynMarkerResult {
                    parent: parent.clone(),
                    before: None,
                    initial: None,
                    multi,
                });
            }
        }
    }
}

/// Instantiate a template by creating nodes to match the template structure. Returns the root node
/// along with a list of flagged nodes and dynamic markers.
pub fn instantiate_template_universal<G: GenericNodeElements>(
    template: &Template,
    opts: InstantiateUniversalOpts,
) -> TemplateResult<G> {
    let mut flagged_nodes = Vec::new();
    let mut dyn_markers = Vec::new();
    let root = instantiate_element_universal_impl(
        &template.shape,
        &mut flagged_nodes,
        &mut dyn_markers,
        opts,
    );
    TemplateResult {
        root,
        flagged_nodes,
        dyn_markers,
    }
}

//! Rendering backend for the DOM with hydration support.

use std::fmt;
use std::hash::{Hash, Hasher};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Node;

use crate::generic_node::{DomNode, GenericNode, Html};
use crate::reactive::{create_scope, ReactiveScope};
use crate::utils::hydrate::web::get_next_element;
use crate::utils::hydrate::{hydration_completed, with_hydration_context};
use crate::utils::render::insert;
use crate::view::View;

use super::dom_node::NodeId;

/// Rendering backend for the DOM with hydration support.
///
/// _This API requires the following crate features to be activated: `experimental-hydrate`, `dom`_
#[derive(Clone)]
pub struct HydrateNode {
    node: DomNode,
}

impl HydrateNode {
    pub fn inner_element(&self) -> Node {
        self.node.inner_element()
    }

    pub fn unchecked_into<T: JsCast>(self) -> T {
        self.node.unchecked_into()
    }

    pub(super) fn get_node_id(&self) -> NodeId {
        self.node.get_node_id()
    }

    pub fn from_web_sys(node: Node) -> Self {
        Self {
            node: DomNode::from_web_sys(node),
        }
    }
}

impl PartialEq for HydrateNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Eq for HydrateNode {}

impl Hash for HydrateNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_node_id().hash(state);
    }
}

impl AsRef<JsValue> for HydrateNode {
    fn as_ref(&self) -> &JsValue {
        self.node.as_ref()
    }
}

impl From<HydrateNode> for JsValue {
    fn from(node: HydrateNode) -> Self {
        JsValue::from(node.node)
    }
}

impl fmt::Debug for HydrateNode {
    /// Prints outerHtml of [`Node`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl GenericNode for HydrateNode {
    type EventType = web_sys::Event;
    const USE_HYDRATION_CONTEXT: bool = true;
    const CLIENT_SIDE_HYDRATION: bool = true;

    /// When hydrating, instead of creating a new node, this will attempt to hydrate an existing
    /// node.
    fn element(tag: &str) -> Self {
        let el = get_next_element();
        if let Some(el) = el {
            Self {
                node: DomNode::from_web_sys(el.into()),
            }
        } else {
            Self {
                node: DomNode::element(tag),
            }
        }
    }

    /// When hydrating, instead of creating a new node, this will attempt to hydrate an existing
    /// node.
    fn text_node(text: &str) -> Self {
        // TODO
        Self {
            node: DomNode::text_node(text),
        }
    }

    fn marker() -> Self {
        // TODO
        Self {
            node: DomNode::marker(),
        }
    }

    fn marker_with_text(text: &str) -> Self {
        // TODO
        Self {
            node: DomNode::marker_with_text(text),
        }
    }

    #[inline]
    fn set_attribute(&self, name: &str, value: &str) {
        self.node.set_attribute(name, value);
    }

    #[inline]
    fn remove_attribute(&self, name: &str) {
        self.node.remove_attribute(name);
    }

    #[inline]
    fn set_class_name(&self, value: &str) {
        self.node.set_class_name(value);
    }

    #[inline]
    fn add_class(&self, class: &str) {
        self.node.add_class(class);
    }

    #[inline]
    fn remove_class(&self, class: &str) {
        self.node.remove_class(class);
    }

    #[inline]
    fn set_property(&self, name: &str, value: &JsValue) {
        self.node.set_property(name, value);
    }

    #[inline]
    fn remove_property(&self, name: &str) {
        self.node.remove_property(name);
    }

    #[inline]
    fn append_child(&self, child: &Self) {
        if hydration_completed() {
            // Do not append nodes during hydration as that will result in duplicate text nodes.
            self.node.append_child(&child.node);
        }
    }

    #[inline]
    fn first_child(&self) -> Option<Self> {
        self.node.first_child().map(|node| Self { node })
    }

    #[inline]
    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        self.node
            .insert_child_before(&new_node.node, reference_node.map(|node| &node.node));
    }

    #[inline]
    fn remove_child(&self, child: &Self) {
        self.node.remove_child(&child.node);
    }

    #[inline]
    fn replace_child(&self, old: &Self, new: &Self) {
        self.node.replace_child(&old.node, &new.node);
    }

    #[inline]
    fn insert_sibling_before(&self, child: &Self) {
        self.node.insert_sibling_before(&child.node);
    }

    #[inline]
    fn parent_node(&self) -> Option<Self> {
        self.node.parent_node().map(|node| Self { node })
    }

    #[inline]
    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self { node })
    }

    #[inline]
    fn remove_self(&self) {
        self.node.remove_self();
    }

    #[inline]
    fn event(&self, name: &str, handler: Box<dyn Fn(Self::EventType)>) {
        self.node.event(name, handler);
    }

    #[inline]
    fn update_inner_text(&self, text: &str) {
        self.node.update_inner_text(text);
    }

    #[inline]
    fn dangerously_set_inner_html(&self, html: &str) {
        self.node.dangerously_set_inner_html(html);
    }

    #[inline]
    fn clone_node(&self) -> Self {
        Self {
            node: self.node.clone_node(),
        }
    }
}

impl Html for HydrateNode {
    const IS_BROWSER: bool = true;
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration). Alias for [`hydrate_to`] with `parent` being the `<body>` tag.
///
/// For rendering without hydration, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `experimental-hydrate`, `dom`_
pub fn hydrate(template: impl FnOnce() -> View<HydrateNode>) {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    hydrate_to(template, &document.body().unwrap_throw());
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration). For rendering under the `<body>` tag, use [`hydrate_to`] instead.
///
/// For rendering without hydration, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `experimental-hydrate`, `dom`_
pub fn hydrate_to(template: impl FnOnce() -> View<HydrateNode>, parent: &Node) {
    let scope = hydrate_get_scope(template, parent);

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));
}

/// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
/// This function is intended to be used for injecting an ephemeral sycamore view into a
/// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
/// modal is closed).
///
/// _This API requires the following crate features to be activated: `experimental-hydrate`, `dom`_
#[must_use = "please hold onto the ReactiveScope until you want to clean things up, or use render_to() instead"]
pub fn hydrate_get_scope(
    template: impl FnOnce() -> View<HydrateNode>,
    parent: &Node,
) -> ReactiveScope {
    create_scope(|| {
        insert(
            &HydrateNode::from_web_sys(parent.clone()),
            with_hydration_context(template),
            None,
            None,
            false,
        );
    })
}

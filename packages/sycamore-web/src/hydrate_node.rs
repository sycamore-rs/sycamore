//! Rendering backend for the DOM with hydration support.

use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};

use sycamore_core::event::{EventDescriptor, EventHandler};
use sycamore_core::generic_node::{
    GenericNode, GenericNodeElements, SycamoreElement, Template, TemplateResult,
};
use sycamore_core::hydrate::{hydration_completed, with_hydration_context};
use sycamore_core::render::insert;
use sycamore_core::view::View;
use sycamore_reactive::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Node;

use crate::dom_node::{DomNode, NodeId};
use crate::dom_node_template::{
    add_new_cached_template, execute_walk, try_get_cached_template, WalkResult,
};
use crate::hydrate::get_next_element;
use crate::Html;

/// Rendering backend for the DOM with hydration support.
///
/// _This API requires the following crate features to be activated: `hydrate`, `dom`_
#[derive(Clone)]
pub struct HydrateNode {
    node: DomNode,
}

impl HydrateNode {
    /// Cast the underlying [`web_sys::Node`] using [`JsCast`].
    pub fn unchecked_into<T: JsCast>(self) -> T {
        self.node.unchecked_into()
    }

    /// Get the [`NodeId`] for the node.
    pub(super) fn get_node_id(&self) -> NodeId {
        self.node.get_node_id()
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
    type AnyEventData = JsValue;
    type PropertyType = JsValue;

    const USE_HYDRATION_CONTEXT: bool = true;
    const CLIENT_SIDE_HYDRATION: bool = true;

    fn text_node(text: Cow<'static, str>) -> Self {
        Self {
            node: DomNode::text_node(text),
        }
    }

    fn marker() -> Self {
        Self {
            node: DomNode::marker(),
        }
    }

    fn marker_with_text(text: Cow<'static, str>) -> Self {
        Self {
            node: DomNode::marker_with_text(text),
        }
    }

    #[inline]
    fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>) {
        self.node.set_attribute(name, value);
    }

    #[inline]
    fn remove_attribute(&self, name: Cow<'static, str>) {
        self.node.remove_attribute(name);
    }

    #[inline]
    fn set_class_name(&self, value: Cow<'static, str>) {
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
        self.node.append_child(&child.node);
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
    fn untyped_event<'a>(
        &self,
        cx: Scope<'a>,
        event: Cow<'_, str>,
        handler: Box<dyn FnMut(Self::AnyEventData) + 'a>,
    ) {
        self.node.untyped_event(cx, event, handler);
    }

    #[inline]
    fn update_inner_text(&self, text: Cow<'static, str>) {
        self.node.update_inner_text(text);
    }

    #[inline]
    fn dangerously_set_inner_html(&self, html: Cow<'static, str>) {
        self.node.dangerously_set_inner_html(html);
    }

    #[inline]
    fn clone_node(&self) -> Self {
        Self {
            node: self.node.clone_node(),
        }
    }
}

impl GenericNodeElements for HydrateNode {
    /// When hydrating, instead of creating a new node, this will attempt to hydrate an existing
    /// node.
    fn element<T: SycamoreElement>() -> Self {
        let el = get_next_element();
        if let Some(el) = el {
            // If in debug mode, check that the hydrate element has the same tag as the argument.
            #[cfg(debug_assertions)]
            if T::TAG_NAME.to_ascii_lowercase() != el.tag_name().to_ascii_lowercase() {
                // Get the hydration key of the expected element.
                let mut hk = sycamore_core::hydrate::get_current_id().unwrap();
                hk.1 -= 1; // Decrement the element id because we called get_next_id previously.
                panic!("hydration error, mismatched element tag\nexpected {}, found {}\noccurred at element with hydration key {}.{}",
                    T::TAG_NAME,
                    el.tag_name().to_ascii_lowercase(),
                    hk.0, hk.1
                );
            }

            Self {
                node: DomNode::from_web_sys(el.into()),
            }
        } else {
            Self {
                node: DomNode::element::<T>(),
            }
        }
    }

    /// When hydrating, instead of creating a new node, this will attempt to hydrate an existing
    /// node.
    fn element_from_tag(tag: Cow<'static, str>) -> Self {
        let el = get_next_element();
        if let Some(el) = el {
            // If in debug mode, check that the hydrate element has the same tag as the argument.
            #[cfg(debug_assertions)]
            if tag != el.tag_name().to_ascii_lowercase() {
                // Get the hydration key of the expected element.
                let mut hk = sycamore_core::hydrate::get_current_id().unwrap();
                hk.1 -= 1; // Decrement the element id because we called get_next_id previously.
                panic!("hydration error, mismatched element tag\nexpected {}, found {}\noccurred at element with hydration key {}.{}",
                    tag,
                    el.tag_name().to_ascii_lowercase(),
                    hk.0, hk.1
                );
            }

            Self {
                node: DomNode::from_web_sys(el.into()),
            }
        } else {
            Self {
                node: DomNode::element_from_tag(tag),
            }
        }
    }

    fn element_from_tag_namespace(tag: Cow<'static, str>, namespace: Cow<'static, str>) -> Self {
        let el = get_next_element();
        if let Some(el) = el {
            // If in debug mode, check that the hydrate element has the same tag as the argument.
            #[cfg(debug_assertions)]
            if tag != el.tag_name().to_ascii_lowercase() {
                // Get the hydration key of the expected element.
                let mut hk = sycamore_core::hydrate::get_current_id().unwrap();
                hk.1 -= 1; // Decrement the element id because we called get_next_id previously.
                panic!("hydration error, mismatched element tag\nexpected {}, found {}\noccurred at element with hydration key {}.{}",
                    tag,
                    el.tag_name().to_ascii_lowercase(),
                    hk.0, hk.1
                );
            }

            Self {
                node: DomNode::from_web_sys(el.into()),
            }
        } else {
            Self {
                node: DomNode::element_from_tag_namespace(tag, namespace),
            }
        }
    }

    /// For performance reasons, we will render this template to an HTML string and then cache it.
    ///
    /// We can then cerate an HTML template element and clone it to create a new instance.
    fn instantiate_template(template: &Template) -> TemplateResult<HydrateNode> {
        if let Some(cached) = try_get_cached_template(template.id) {
            let hydrate_mode = !hydration_completed();

            let root = if hydrate_mode {
                get_next_element()
                    .expect("node with hydration key not found")
                    .into()
            } else {
                cached.clone_template_content()
            };

            // Execute the walk sequence.
            let WalkResult {
                flagged_nodes,
                dyn_markers,
            } = execute_walk(&cached.walk, &root, hydrate_mode);

            TemplateResult {
                root: HydrateNode::from_web_sys(root),
                flagged_nodes,
                dyn_markers,
            }
        } else {
            add_new_cached_template(template);
            // Now that the cached template has been created, we can use it.
            Self::instantiate_template(template)
        }
    }
}

impl Html for HydrateNode {
    const IS_BROWSER: bool = true;

    fn to_web_sys(&self) -> web_sys::Node {
        self.node.to_web_sys()
    }

    fn from_web_sys(node: Node) -> Self {
        Self {
            node: DomNode::from_web_sys(node),
        }
    }
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration). Alias for [`hydrate_to`] with `parent` being the `<body>` tag.
///
/// For rendering without hydration, use [`render`](super::render) instead.
///
/// _This API requires the following crate features to be activated: `hydrate`, `dom`_
pub fn hydrate(view: impl FnOnce(Scope<'_>) -> View<HydrateNode>) {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    hydrate_to(view, &document.body().unwrap_throw());
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration). For rendering under the `<body>` tag, use [`hydrate_to`] instead.
///
/// For rendering without hydration, use [`render`](super::render) instead.
///
/// _This API requires the following crate features to be activated: `hydrate`, `dom`_
pub fn hydrate_to(view: impl FnOnce(Scope<'_>) -> View<HydrateNode>, parent: &Node) {
    // Do not call the destructor function, effectively leaking the scope.
    let _ = hydrate_get_scope(view, parent);
}

/// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
/// This function is intended to be used for injecting an ephemeral sycamore view into a
/// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
/// modal is closed).
///
/// _This API requires the following crate features to be activated: `hydrate`, `dom`_
#[must_use = "please hold onto the ReactiveScope until you want to clean things up, or use render_to() instead"]
pub fn hydrate_get_scope<'a>(
    view: impl FnOnce(Scope<'_>) -> View<HydrateNode> + 'a,
    parent: &'a Node,
) -> ScopeDisposer<'a> {
    // Get children from parent into a View to set as the initial node value.
    let mut children = Vec::new();
    let child_nodes = parent.child_nodes();
    for i in 0..child_nodes.length() {
        children.push(child_nodes.get(i).unwrap());
    }
    let children = children
        .into_iter()
        .map(|x| View::new_node(HydrateNode::from_web_sys(x)))
        .collect::<Vec<_>>();

    create_scope(|cx| {
        insert(
            cx,
            &HydrateNode::from_web_sys(parent.clone()),
            with_hydration_context(|| view(cx)),
            Some(View::new_fragment(children)),
            None,
            false,
        );
    })
}

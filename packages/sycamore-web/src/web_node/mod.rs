//! The web rendering backend node type.

#[cfg(feature = "dom")]
pub mod dom;
#[cfg(feature = "ssr")]
pub mod ssr;

use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};

use sycamore_core::generic_node::{GenericNode, GenericNodeElements};
use sycamore_core::view::View;
use sycamore_reactive::Scope;
use wasm_bindgen::{JsCast, JsValue};

use crate::hydrate::{
    get_next_markers, is_hydrating, use_hydration_ctx, use_hydration_state, HydrationKey,
};
use crate::render::{get_render_env, RenderEnv};

pub struct WebNode(WebNodeInner);

impl WebNode {
    /// Create a new [`WebNode`] from a [`web_sys::Node`].
    ///
    /// **Important**: This will create a DOM node regardless of what the actual environment is.
    /// Note that it does not make any sense to create a SSR node from a real raw DOM node.
    #[cfg(feature = "dom")]
    pub fn from_web_sys(root: web_sys::Node) -> Self {
        Self(WebNodeInner::Dom(dom::DomNode::from_web_sys(root)))
    }

    /// Get the underlying [`web_sys::Node`].
    ///
    /// # Panics
    ///
    /// This will panic if the node is a SSR node.
    #[cfg(feature = "dom")]
    pub fn to_web_sys(&self) -> web_sys::Node {
        self.as_dom_node()
            .expect("can not convert a SSR node to a web_sys::Node")
            .to_web_sys()
    }

    /// Get the underlying [`dom::DomNode`] or `None` if it is a SSR node.
    #[cfg(feature = "dom")]
    pub fn as_dom_node(&self) -> Option<&dom::DomNode> {
        match &self.0 {
            WebNodeInner::Dom(node) => Some(node),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(_) => None,
        }
    }

    /// Get the underlying [`ssr::SsrNode`] or `None` if it is a DOM node.
    #[cfg(feature = "ssr")]
    pub fn as_ssr_node(&self) -> Option<&ssr::SsrNode> {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(_) => None,
            WebNodeInner::Ssr(node) => Some(node),
        }
    }

    /// Create a new [`WebNode`] from a [`dom::DomNode`].
    #[cfg(feature = "dom")]
    pub fn from_dom_node(node: dom::DomNode) -> Self {
        Self(WebNodeInner::Dom(node))
    }

    /// Create a new [`WebNode`] from a [`ssr::SsrNode`].
    #[cfg(feature = "ssr")]
    pub fn from_ssr_node(node: ssr::SsrNode) -> Self {
        Self(WebNodeInner::Ssr(node))
    }
}

/// Internal implementation of [`WebNode`].
enum WebNodeInner {
    #[cfg(feature = "dom")]
    Dom(dom::DomNode),
    #[cfg(feature = "ssr")]
    Ssr(ssr::SsrNode),
}

#[allow(unreachable_patterns)]
impl GenericNode for WebNode {
    fn text_node(cx: Scope, text: Cow<'static, str>) -> Self {
        match get_render_env(cx) {
            #[cfg(feature = "dom")]
            RenderEnv::Dom => Self(WebNodeInner::Dom(dom::DomNode::text_node(text))),
            #[cfg(feature = "ssr")]
            RenderEnv::Ssr => Self(WebNodeInner::Ssr(ssr::SsrNode::text_node(text))),
            _ => panic!("feature not enabled for render env"),
        }
    }

    fn marker_with_text(cx: Scope, text: Cow<'static, str>) -> Self {
        match get_render_env(cx) {
            #[cfg(feature = "dom")]
            RenderEnv::Dom => Self(WebNodeInner::Dom(dom::DomNode::marker_with_text(text))),
            #[cfg(feature = "ssr")]
            RenderEnv::Ssr => Self(WebNodeInner::Ssr(ssr::SsrNode::marker_with_text(text))),
            _ => panic!("feature not enabled for render env"),
        }
    }

    fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.set_attribute(name, value),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.set_attribute(name, value),
        }
    }

    fn remove_attribute(&self, name: Cow<'static, str>) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.remove_attribute(name),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.remove_attribute(name),
        }
    }

    fn set_class_name(&self, value: Cow<'static, str>) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.set_class_name(value),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.set_class_name(value),
        }
    }

    fn append_child(&self, child: &Self) {
        match (&self.0, &child.0) {
            #[cfg(feature = "dom")]
            (WebNodeInner::Dom(node), WebNodeInner::Dom(child)) => node.append_child(child),
            #[cfg(feature = "ssr")]
            (WebNodeInner::Ssr(node), WebNodeInner::Ssr(child)) => node.append_child(child),
            _ => panic!("cannot intermix SSR and DOM nodes"),
        }
    }

    fn first_child(&self) -> Option<Self> {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.first_child().map(Self::from_dom_node),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.first_child().map(Self::from_ssr_node),
        }
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        match (&self.0, &new_node.0) {
            #[cfg(feature = "dom")]
            (WebNodeInner::Dom(node), WebNodeInner::Dom(new_node)) => {
                node.insert_child_before(new_node, reference_node.and_then(Self::as_dom_node))
            }
            #[cfg(feature = "ssr")]
            (WebNodeInner::Ssr(node), WebNodeInner::Ssr(new_node)) => {
                node.insert_child_before(new_node, reference_node.and_then(Self::as_ssr_node))
            }
            _ => panic!("cannot intermix SSR and DOM nodes"),
        }
    }

    fn remove_child(&self, child: &Self) {
        match (&self.0, &child.0) {
            #[cfg(feature = "dom")]
            (WebNodeInner::Dom(node), WebNodeInner::Dom(child)) => node.remove_child(child),
            #[cfg(feature = "ssr")]
            (WebNodeInner::Ssr(node), WebNodeInner::Ssr(child)) => node.remove_child(child),
            _ => panic!("cannot intermix SSR and DOM nodes"),
        }
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        match (&self.0, &old.0, &new.0) {
            #[cfg(feature = "dom")]
            (WebNodeInner::Dom(node), WebNodeInner::Dom(old), WebNodeInner::Dom(new)) => {
                node.replace_child(old, new)
            }
            #[cfg(feature = "ssr")]
            (WebNodeInner::Ssr(node), WebNodeInner::Ssr(old), WebNodeInner::Ssr(new)) => {
                node.replace_child(old, new)
            }
            _ => panic!("cannot intermix SSR and DOM nodes"),
        }
    }

    fn insert_sibling_before(&self, child: &Self) {
        match (&self.0, &child.0) {
            #[cfg(feature = "dom")]
            (WebNodeInner::Dom(node), WebNodeInner::Dom(child)) => {
                node.insert_sibling_before(child)
            }
            #[cfg(feature = "ssr")]
            (WebNodeInner::Ssr(node), WebNodeInner::Ssr(child)) => {
                node.insert_sibling_before(child)
            }
            _ => panic!("cannot intermix SSR and DOM nodes"),
        }
    }

    fn parent_node(&self) -> Option<Self> {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.parent_node().map(Self::from_dom_node),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.parent_node().map(Self::from_ssr_node),
        }
    }

    fn next_sibling(&self) -> Option<Self> {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.next_sibling().map(Self::from_dom_node),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.next_sibling().map(Self::from_ssr_node),
        }
    }

    fn remove_self(&self) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.remove_self(),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.remove_self(),
        }
    }

    fn update_inner_text(&self, text: Cow<'static, str>) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.update_inner_text(text),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.update_inner_text(text),
        }
    }

    fn dangerously_set_inner_html(&self, html: Cow<'static, str>) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.dangerously_set_inner_html(html),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.dangerously_set_inner_html(html),
        }
    }

    fn clone_node(&self) -> Self {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => Self::from_dom_node(node.clone_node()),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => Self::from_ssr_node(node.clone_node()),
        }
    }

    fn finish_element(&mut self, _cx: Scope, _is_dyn: bool) {
        #[cfg(feature = "hydrate")]
        match &mut self.0 {
            // If we are in SSR mode, add a `data-hk` attribute to the element.
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => {
                if let Some(hk) = node.0.hk.get() {
                    node.set_attribute("data-hk".into(), hk.to_string().into());
                }
            }
            _ => {}
        }
    }

    fn get_next_element(_cx: Scope, f: impl Fn() -> Self) -> Self {
        // If we are hydrating on the client-side, get the next element from the hydration state.
        // Otherwise, call f.
        #[cfg(feature = "hydrate")]
        match get_render_env(_cx) {
            #[cfg(feature = "dom")]
            RenderEnv::Dom => {
                if is_hydrating(_cx) {
                    let hk = use_hydration_state(_cx).next_key();
                    let h_ctx = use_hydration_ctx(_cx);
                    let node = h_ctx
                        .get_element_by_key(hk)
                        .cloned()
                        .map(Self::from_web_sys);
                    match node {
                        Some(node) => node,
                        None => {
                            #[cfg(debug_assertions)]
                            web_sys::console::warn_1(
                                &format!("No element with hydration key {hk}. Skipping.").into(),
                            );
                            f()
                        }
                    }
                } else {
                    f()
                }
            }
            #[cfg(feature = "ssr")]
            RenderEnv::Ssr => {
                let hk = if is_hydrating(_cx) {
                    // Increment the hydration key to stay in sync with client.
                    Some(use_hydration_state(_cx).next_key())
                } else {
                    None
                };
                let el = f();
                el.as_ssr_node().unwrap().0.hk.set(hk);
                el
            }
        }
        #[cfg(not(feature = "hydrate"))]
        f()
    }

    fn builder_insert(&self, cx: Scope, view: View<Self>) {
        let is_hydrating = is_hydrating(cx);
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(_) => {
                // If it is a static, we don't need to insert it again.
                // Otherwise, find the start and end markers and insert the view between them.
                let initial = if is_hydrating {
                    if view.is_node() {
                        return;
                    } else {
                        let initial = get_next_markers(&self.to_web_sys().unchecked_into());
                        initial.map(|nodes| {
                            let nodes = nodes
                                .into_iter()
                                .map(Self::from_web_sys)
                                .map(View::new_node)
                                .collect();
                            View::new_fragment(nodes)
                        })
                    }
                } else {
                    None
                };
                sycamore_core::render::insert(cx, self, view, initial, None, true);
            }
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(_) => {
                if view.is_node() {
                    sycamore_core::render::insert(cx, self, view, None, None, true);
                } else {
                    // Only insert start tag if we are hydrating.
                    // Start tags are useless outside of hydration.
                    if is_hydrating {
                        let start_tag = Self::marker_with_text(cx, "#".into());
                        self.append_child(&start_tag);
                    }
                    let end_tag = Self::marker_with_text(cx, "/".into());
                    self.append_child(&end_tag);

                    sycamore_core::render::insert(cx, self, view, None, Some(&end_tag), true);
                }
            }
        }
    }

    fn component_scope(cx: Scope, f: impl FnOnce() -> View<Self>) -> View<Self> {
        if is_hydrating(cx) {
            let h_state = use_hydration_state(cx);
            h_state.enter_component(f)
        } else {
            f()
        }
    }
}

#[allow(unreachable_patterns)]
impl GenericNodeElements for WebNode {
    type AnyEventData = JsValue;

    fn element_from_tag(cx: Scope, tag: Cow<'static, str>) -> Self {
        match get_render_env(cx) {
            #[cfg(feature = "dom")]
            RenderEnv::Dom => Self::from_dom_node(dom::DomNode::element_from_tag(tag)),
            #[cfg(feature = "ssr")]
            RenderEnv::Ssr => {
                Self::from_ssr_node(ssr::SsrNode::element_from_tag(tag, HydrationKey::null())) // FIXME: hk null
            }
            _ => panic!("feature not enabled for render env"),
        }
    }

    fn element_from_tag_namespace(
        cx: Scope,
        tag: Cow<'static, str>,
        namespace: Cow<'static, str>,
    ) -> Self {
        match get_render_env(cx) {
            #[cfg(feature = "dom")]
            RenderEnv::Dom => {
                Self::from_dom_node(dom::DomNode::element_from_tag_namespace(tag, namespace))
            }
            #[cfg(feature = "ssr")]
            RenderEnv::Ssr => Self::from_ssr_node(ssr::SsrNode::element_from_tag_namespace(
                tag,
                namespace,
                HydrationKey::null(),
            )), // FIXME: hk null
            _ => panic!("feature not enabled for render env"),
        }
    }

    fn add_event_listener<'a>(
        &self,
        cx: Scope<'a>,
        name: &str,
        listener: Box<dyn FnMut(Self::AnyEventData) + 'a>,
    ) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.add_event_listener(cx, name, listener),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.add_event_listener(cx, name, listener),
        }
    }
}

impl WebNode {
    pub fn set_property(&self, name: &str, value: JsValue) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.set_property(name, value),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.set_property(name, value),
        }
    }
}

impl PartialEq for WebNode {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            #[cfg(feature = "dom")]
            (WebNodeInner::Dom(node), WebNodeInner::Dom(other)) => node == other,
            #[cfg(feature = "ssr")]
            (WebNodeInner::Ssr(node), WebNodeInner::Ssr(other)) => node == other,
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}
impl Eq for WebNode {}

impl Clone for WebNode {
    fn clone(&self) -> Self {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => Self::from_dom_node(node.clone()),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => Self::from_ssr_node(node.clone()),
        }
    }
}

impl Hash for WebNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.hash(state),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.hash(state),
        }
    }
}

impl fmt::Debug for WebNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(node) => node.fmt(f),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => node.fmt(f),
        }
    }
}

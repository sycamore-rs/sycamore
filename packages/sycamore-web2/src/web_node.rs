//! The web rendering backend node type.

#[cfg(feature = "dom")]
pub mod dom;
#[cfg(feature = "ssr")]
pub mod ssr;

use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};

use sycamore_core2::generic_node::{GenericNode, GenericNodeElements, SycamoreElement};
use sycamore_reactive::Scope;

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

    /// Get the underlying [`dom::DomNode`] or `None` if it is a SSR node.
    #[cfg(feature = "dom")]
    pub fn as_dom_node(&self) -> Option<&dom::DomNode> {
        match &self.0 {
            WebNodeInner::Dom(node) => Some(node),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(_) => None,
        }
    }

    #[cfg(feature = "ssr")]
    pub fn as_ssr_node(&self) -> Option<&ssr::SsrNode> {
        match &self.0 {
            #[cfg(feature = "dom")]
            WebNodeInner::Dom(_) => None,
            WebNodeInner::Ssr(node) => Some(node),
        }
    }

    #[cfg(feature = "dom")]
    pub fn from_dom_node(node: dom::DomNode) -> Self {
        Self(WebNodeInner::Dom(node))
    }

    #[cfg(feature = "ssr")]
    pub fn from_ssr_node(node: ssr::SsrNode) -> Self {
        Self(WebNodeInner::Ssr(node))
    }
}

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
}

#[allow(unreachable_patterns)]
impl GenericNodeElements for WebNode {
    fn element<T: SycamoreElement>(cx: Scope) -> Self {
        match get_render_env(cx) {
            #[cfg(feature = "dom")]
            RenderEnv::Dom => Self::from_dom_node(dom::DomNode::element::<T>()),
            #[cfg(feature = "ssr")]
            RenderEnv::Ssr => Self::from_ssr_node(ssr::SsrNode::element::<T>()),
            _ => panic!("feature not enabled for render env"),
        }
    }

    fn element_from_tag(cx: Scope, tag: Cow<'static, str>) -> Self {
        match get_render_env(cx) {
            #[cfg(feature = "dom")]
            RenderEnv::Dom => Self::from_dom_node(dom::DomNode::element_from_tag(tag)),
            #[cfg(feature = "ssr")]
            RenderEnv::Ssr => Self::from_ssr_node(ssr::SsrNode::element_from_tag(tag)),
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
            RenderEnv::Ssr => {
                Self::from_ssr_node(ssr::SsrNode::element_from_tag_namespace(tag, namespace))
            }
            _ => panic!("feature not enabled for render env"),
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
            WebNodeInner::Dom(node) => Self::from_dom_node(node.clone_node()),
            #[cfg(feature = "ssr")]
            WebNodeInner::Ssr(node) => Self::from_ssr_node(node.clone_node()),
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

//! Internal implementation for the browser DOM.

use std::borrow::Cow;
use std::cell::Cell;
use std::fmt;
use std::hash::{Hash, Hasher};

use sycamore_core2::generic_node::SycamoreElement;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{intern, JsCast};
use web_sys::{Comment, Element, Node, Text};

use crate::document;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Node)]
    pub(super) type NodeWithId;
    #[wasm_bindgen(method, getter, js_name = "$$$nodeId")]
    pub fn node_id(this: &NodeWithId) -> Option<usize>;
    #[wasm_bindgen(method, setter, js_name = "$$$nodeId")]
    pub fn set_node_id(this: &NodeWithId, id: usize);

    #[wasm_bindgen(extends = Element)]
    type ElementTrySetClassName;
    #[wasm_bindgen(method, catch, setter, js_name = "className")]
    fn try_set_class_name(this: &ElementTrySetClassName, class_name: &str) -> Result<(), JsValue>;
}

/// An unique id for every node.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct NodeId(pub usize);

impl NodeId {
    pub fn new_with_node(node: &Node) -> Self {
        thread_local!(static NODE_ID_COUNTER: Cell<usize> = Cell::new(1)); // 0 is reserved for default value.

        let id = NODE_ID_COUNTER.with(|x| {
            let tmp = x.get();
            x.set(tmp + 1);
            tmp
        });
        node.unchecked_ref::<NodeWithId>().set_node_id(id);
        Self(id)
    }
}

/// Rendering backend for the DOM.
#[derive(Clone)]
pub struct DomNode {
    id: Cell<NodeId>,
    node: Node,
}

impl DomNode {
    /// Cast the underlying [`web_sys::Node`] using [`JsCast`].
    pub fn unchecked_into<T: JsCast>(self) -> T {
        self.node.unchecked_into()
    }

    /// Get the [`NodeId`] for the node.
    pub(super) fn get_node_id(&self) -> NodeId {
        if self.id.get() == NodeId(0) {
            // self.id not yet initialized.
            if let Some(id) = self.node.unchecked_ref::<NodeWithId>().node_id() {
                self.id.set(NodeId(id));
            } else {
                self.id.set(NodeId::new_with_node(&self.node));
            }
        }
        self.id.get()
    }

    pub fn from_web_sys(node: Node) -> Self {
        Self {
            id: Cell::new(NodeId(0)),
            node,
        }
    }
}

impl PartialEq for DomNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Eq for DomNode {}

impl Hash for DomNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_node_id().hash(state);
    }
}

impl AsRef<JsValue> for DomNode {
    fn as_ref(&self) -> &JsValue {
        self.node.as_ref()
    }
}

impl From<DomNode> for JsValue {
    fn from(node: DomNode) -> Self {
        (*node.node).clone().into()
    }
}

impl fmt::Debug for DomNode {
    /// Prints outerHtml of [`Node`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let outer_html = if let Some(element) = self.node.dyn_ref::<Element>() {
            element.outer_html()
        } else if let Some(text) = self.node.dyn_ref::<Text>() {
            text.text_content().unwrap_or_default()
        } else if let Some(comment) = self.node.dyn_ref::<Comment>() {
            format!("<!--{}-->", comment.text_content().unwrap_or_default())
        } else {
            self.node.to_string().as_string().unwrap_throw()
        };
        f.debug_tuple("DomNode").field(&outer_html).finish()
    }
}

/// `GenericNode` methods.
impl DomNode {
    pub fn text_node(text: Cow<'static, str>) -> Self {
        let node = document().create_text_node(&text).into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    pub fn marker_with_text(text: Cow<'static, str>) -> Self {
        let node = document().create_comment(&text).into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    pub fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>) {
        self.node
            .unchecked_ref::<Element>()
            .set_attribute(intern(&name), &value)
            .unwrap_throw();
    }

    pub fn remove_attribute(&self, name: Cow<'static, str>) {
        self.node
            .unchecked_ref::<Element>()
            .remove_attribute(intern(&name))
            .unwrap_throw();
    }

    pub fn set_class_name(&self, value: Cow<'static, str>) {
        if self
            .node
            .unchecked_ref::<ElementTrySetClassName>()
            .try_set_class_name(&value)
            .is_err()
        {
            // Node is a SVG element.
            self.node
                .unchecked_ref::<Element>()
                .set_attribute("class", &value)
                .unwrap_throw();
        }
    }

    pub fn append_child(&self, child: &Self) {
        self.node.append_child(&child.node).unwrap_throw();
    }

    pub fn first_child(&self) -> Option<Self> {
        self.node.first_child().map(|node| Self {
            id: Default::default(),
            node,
        })
    }

    pub fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        self.node
            .insert_before(&new_node.node, reference_node.map(|n| &n.node))
            .unwrap_throw();
    }

    pub fn remove_child(&self, child: &Self) {
        self.node.remove_child(&child.node).unwrap_throw();
    }

    pub fn replace_child(&self, old: &Self, new: &Self) {
        self.node.replace_child(&new.node, &old.node).unwrap_throw();
    }

    pub fn insert_sibling_before(&self, child: &Self) {
        self.node
            .unchecked_ref::<Element>()
            .before_with_node_1(&child.node)
            .unwrap_throw();
    }

    pub fn parent_node(&self) -> Option<Self> {
        self.node.parent_node().map(|node| Self {
            id: Default::default(),
            node,
        })
    }

    pub fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self {
            id: Default::default(),
            node,
        })
    }

    pub fn remove_self(&self) {
        self.node.unchecked_ref::<Element>().remove();
    }

    pub fn update_inner_text(&self, text: Cow<'static, str>) {
        self.node.set_text_content(Some(&text));
    }

    pub fn dangerously_set_inner_html(&self, html: Cow<'static, str>) {
        self.node.unchecked_ref::<Element>().set_inner_html(&html);
    }

    pub fn clone_node(&self) -> Self {
        Self {
            node: self.node.clone_node_with_deep(true).unwrap_throw(),
            id: Default::default(),
        }
    }
}

/// `GenericNodeElements` methods.
impl DomNode {
    pub fn element<T: SycamoreElement>() -> Self {
        let node = if let Some(ns) = T::NAMESPACE {
            document()
                .create_element_ns(Some(ns), intern(T::TAG_NAME))
                .unwrap_throw()
                .into()
        } else {
            document()
                .create_element(intern(T::TAG_NAME))
                .unwrap_throw()
                .into()
        };
        DomNode {
            id: Default::default(),
            node,
        }
    }

    pub fn element_from_tag(tag: Cow<'static, str>) -> Self {
        let node = document()
            .create_element(intern(&tag))
            .unwrap_throw()
            .into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    pub fn element_from_tag_namespace(
        tag: Cow<'static, str>,
        namespace: Cow<'static, str>,
    ) -> Self {
        let node = document()
            .create_element_ns(Some(intern(&namespace)), intern(&tag))
            .unwrap_throw()
            .into();
        DomNode {
            id: Default::default(),
            node,
        }
    }
}

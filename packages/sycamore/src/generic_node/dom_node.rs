//! Rendering backend for the DOM.

#![allow(clippy::unused_unit)] // TODO: wasm-bindgen bug

use std::cell::Cell;
use std::fmt;
use std::hash::{Hash, Hasher};

use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{intern, JsCast};
use web_sys::{Comment, Document, Element, Node, Text};

use super::SycamoreElement;
use crate::generic_node::{GenericNode, Html};
use crate::reactive::*;
use crate::utils::render::insert;
use crate::view::View;

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

    #[wasm_bindgen(extends = Document)]
    type DocumentCreateTextNodeInt;
    #[wasm_bindgen(method, js_name = "createTextNode")]
    pub fn create_text_node_int(this: &DocumentCreateTextNodeInt, num: i32) -> web_sys::Text;
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
///
/// _This API requires the following crate features to be activated: `dom`_
#[derive(Clone)]
pub struct DomNode {
    id: Cell<NodeId>,
    node: Node,
}

impl DomNode {
    /// Get the underlying [`web_sys::Node`].
    pub fn inner_element(&self) -> Node {
        self.node.clone()
    }

    /// Cast the underlying [`web_sys::Node`] using [`JsCast`].
    pub fn unchecked_into<T: JsCast>(self) -> T {
        self.node.unchecked_into()
    }

    /// Get the [`NodeId`] for the node.
    pub(super) fn get_node_id(&self) -> NodeId {
        if self.id.get().0 == 0 {
            // self.id not yet initialized.
            if let Some(id) = self.node.unchecked_ref::<NodeWithId>().node_id() {
                self.id.set(NodeId(id));
            } else {
                self.id.set(NodeId::new_with_node(&self.node));
            }
        }
        self.id.get()
    }

    /// Create a new [`DomNode`] from a raw [`web_sys::Node`].
    pub fn from_web_sys(node: Node) -> Self {
        Self {
            id: Default::default(),
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

fn document() -> web_sys::Document {
    thread_local! {
        /// Cache document since it is frequently accessed to prevent going through js-interop.
        static DOCUMENT: web_sys::Document = web_sys::window().unwrap_throw().document().unwrap_throw();
    };
    DOCUMENT.with(|document| document.clone())
}

impl GenericNode for DomNode {
    type EventType = web_sys::Event;
    type PropertyType = JsValue;

    fn element<T: SycamoreElement>() -> Self {
        let node = if let Some(ns) = T::NAME_SPACE {
            document()
                .create_element_ns(Some(ns), intern(T::TAG_NAME))
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw()
        } else {
            document()
                .create_element(intern(T::TAG_NAME))
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw()
        };
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn element_from_tag(tag: &str) -> Self {
        let node = document()
            .create_element(intern(tag))
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn text_node(text: &str) -> Self {
        let node = document().create_text_node(text).into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn text_node_int(int: i32) -> Self {
        let node = document()
            .unchecked_into::<DocumentCreateTextNodeInt>()
            .create_text_node_int(int)
            .into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn marker_with_text(text: &str) -> Self {
        let node = document().create_comment(text).into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn set_attribute(&self, name: &str, value: &str) {
        self.node
            .unchecked_ref::<Element>()
            .set_attribute(intern(name), value)
            .unwrap_throw();
    }

    fn remove_attribute(&self, name: &str) {
        self.node
            .unchecked_ref::<Element>()
            .remove_attribute(intern(name))
            .unwrap_throw();
    }

    fn set_class_name(&self, value: &str) {
        if self
            .node
            .unchecked_ref::<ElementTrySetClassName>()
            .try_set_class_name(value)
            .is_err()
        {
            // Node is a SVG element.
            self.node
                .unchecked_ref::<Element>()
                .set_attribute("class", value)
                .unwrap_throw();
        }
    }

    fn add_class(&self, class: &str) {
        let class_list = class.split_ascii_whitespace().collect::<Vec<_>>();
        if class_list.len() == 1 {
            self.node
                .unchecked_ref::<Element>()
                .class_list()
                .add_1(class_list[0])
                .unwrap_throw();
        } else {
            self.node
                .unchecked_ref::<Element>()
                .class_list()
                .add(&class_list.into_iter().map(JsValue::from).collect::<Array>())
                .unwrap_throw();
        }
    }

    fn remove_class(&self, class: &str) {
        let class_list = class.split_ascii_whitespace().collect::<Vec<_>>();
        if class_list.len() == 1 {
            self.node
                .unchecked_ref::<Element>()
                .class_list()
                .remove_1(class_list[0])
                .unwrap_throw();
        } else {
            self.node
                .unchecked_ref::<Element>()
                .class_list()
                .remove(&class_list.into_iter().map(JsValue::from).collect::<Array>())
                .unwrap_throw();
        }
    }

    fn set_property(&self, name: &str, value: &JsValue) {
        assert!(js_sys::Reflect::set(&self.node, &name.into(), value).unwrap_throw());
    }

    fn remove_property(&self, name: &str) {
        assert!(js_sys::Reflect::delete_property(&self.node, &name.into()).unwrap_throw());
    }

    fn append_child(&self, child: &Self) {
        self.node.append_child(&child.node).unwrap_throw();
    }

    fn first_child(&self) -> Option<Self> {
        self.node.first_child().map(|node| Self {
            id: Default::default(),
            node,
        })
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        self.node
            .insert_before(&new_node.node, reference_node.map(|n| n.node.as_ref()))
            .unwrap_throw();
    }

    fn remove_child(&self, child: &Self) {
        self.node.remove_child(&child.node).unwrap_throw();
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        self.node.replace_child(&new.node, &old.node).unwrap_throw();
    }

    fn insert_sibling_before(&self, child: &Self) {
        self.node
            .unchecked_ref::<Element>()
            .before_with_node_1(&child.node)
            .unwrap_throw();
    }

    fn parent_node(&self) -> Option<Self> {
        self.node.parent_node().map(|node| Self {
            id: Default::default(),
            node,
        })
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self {
            id: Default::default(),
            node,
        })
    }

    fn remove_self(&self) {
        self.node.unchecked_ref::<Element>().remove();
    }

    fn event<'a, F: FnMut(Self::EventType) + 'a>(&self, cx: Scope<'a>, name: &str, handler: F) {
        let boxed: Box<dyn FnMut(Self::EventType)> = Box::new(handler);
        // SAFETY: extend lifetime because the closure is dropped when the cx is disposed,
        // preventing the handler from ever being accessed after its lifetime.
        let handler: Box<dyn FnMut(Self::EventType) + 'static> =
            unsafe { std::mem::transmute(boxed) };
        let closure = Closure::wrap(handler);
        self.node
            .add_event_listener_with_callback(intern(name), closure.as_ref().unchecked_ref())
            .unwrap_throw();

        on_cleanup(cx, move || {
            drop(closure);
        });
    }

    fn update_inner_text(&self, text: &str) {
        self.node.set_text_content(Some(text));
    }

    fn dangerously_set_inner_html(&self, html: &str) {
        self.node.unchecked_ref::<Element>().set_inner_html(html);
    }

    fn clone_node(&self) -> Self {
        Self {
            node: self.node.clone_node_with_deep(true).unwrap_throw(),
            id: Default::default(),
        }
    }
}

impl Html for DomNode {
    const IS_BROWSER: bool = true;
}

/// Render a [`View`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render(view: impl FnOnce(Scope<'_>) -> View<DomNode>) {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    render_to(view, &document.body().unwrap_throw());
}

/// Render a [`View`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render_to(view: impl FnOnce(Scope<'_>) -> View<DomNode>, parent: &Node) {
    // Do not call the destructor function, effectively leaking the scope.
    let _ = render_get_scope(view, parent);
}

/// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
/// This function is intended to be used for injecting an ephemeral sycamore view into a
/// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
/// modal is closed).
///
/// It is, however, preferable to have a single call to [`render`] or [`render_to`] at the top level
/// of your app long-term. For rendering a view that will never be unmounted from the dom, use
/// [`render_to`] instead. For rendering under the `<body>` tag, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
#[must_use = "please hold onto the ScopeDisposer until you want to clean things up, or use render_to() instead"]
pub fn render_get_scope<'a>(
    view: impl FnOnce(Scope<'_>) -> View<DomNode> + 'a,
    parent: &'a Node,
) -> ScopeDisposer<'a> {
    create_scope(|cx| {
        insert(
            cx,
            &DomNode::from_web_sys(parent.clone()),
            view(cx),
            None,
            None,
            false,
        );
    })
}

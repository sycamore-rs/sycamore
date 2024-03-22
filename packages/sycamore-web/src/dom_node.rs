//! Rendering backend for the DOM.

use std::borrow::Cow;
use std::cell::Cell;
use std::fmt;
use std::hash::{Hash, Hasher};

use js_sys::Array;
use sycamore_core::generic_node::{
    GenericNode, GenericNodeElements, SycamoreElement, Template, TemplateResult,
};
use sycamore_core::render::insert;
use sycamore_core::view::View;
use sycamore_reactive::*;
use wasm_bindgen::intern;
use wasm_bindgen::prelude::*;
use web_sys::{Comment, Element, Node, Text};

use crate::dom_node_template::{
    add_new_cached_template, execute_walk, try_get_cached_template, WalkResult,
};
use crate::{document, Html};

#[wasm_bindgen]
extern "C" {
    /// Extend [`Node`] with an id field. This is used to make `Node` hashable.
    #[wasm_bindgen(extends = Node)]
    pub(super) type NodeWithId;
    #[wasm_bindgen(method, getter, js_name = "$$$nodeId")]
    pub(crate) fn node_id(this: &NodeWithId) -> Option<usize>;
    #[wasm_bindgen(method, setter, js_name = "$$$nodeId")]
    pub(crate) fn set_node_id(this: &NodeWithId, id: usize);

    /// Extend [`Element`] with a failable `className` setter.
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
        thread_local!(static NODE_ID_COUNTER: Cell<usize> = const { Cell::new(1) }); // 0 is reserved for default value.

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

impl GenericNode for DomNode {
    type AnyEventData = wasm_bindgen::JsValue;
    type PropertyType = JsValue;

    fn text_node(text: Cow<'static, str>) -> Self {
        let node = document().create_text_node(&text).into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn marker_with_text(text: Cow<'static, str>) -> Self {
        let node = document().create_comment(&text).into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>) {
        self.node
            .unchecked_ref::<Element>()
            .set_attribute(intern(&name), &value)
            .unwrap_throw();
    }

    fn remove_attribute(&self, name: Cow<'static, str>) {
        self.node
            .unchecked_ref::<Element>()
            .remove_attribute(intern(&name))
            .unwrap_throw();
    }

    fn set_class_name(&self, value: Cow<'static, str>) {
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
            .insert_before(&new_node.node, reference_node.map(|n| &n.node))
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

    fn untyped_event(
        &self,
        event: Cow<'_, str>,
        mut handler: Box<dyn FnMut(Self::AnyEventData) + 'static>,
    ) {
        let scope = use_global_scope();
        let cb = Closure::new(move |x| {
            scope.run_in(|| handler(x));
        });
        self.node
            .add_event_listener_with_callback(&event, cb.as_ref().unchecked_ref())
            .unwrap_throw();
        on_cleanup(move || drop(cb));
    }

    fn update_inner_text(&self, text: Cow<'static, str>) {
        self.node.set_text_content(Some(&text));
    }

    fn dangerously_set_inner_html(&self, html: Cow<'static, str>) {
        self.node.unchecked_ref::<Element>().set_inner_html(&html);
    }

    fn clone_node(&self) -> Self {
        Self {
            node: self.node.clone_node_with_deep(true).unwrap_throw(),
            id: Default::default(),
        }
    }
}

impl GenericNodeElements for DomNode {
    fn element<T: SycamoreElement>() -> Self {
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

    fn element_from_tag(tag: Cow<'static, str>) -> Self {
        let node = document()
            .create_element(intern(&tag))
            .unwrap_throw()
            .into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn element_from_tag_namespace(tag: Cow<'static, str>, namespace: Cow<'static, str>) -> Self {
        let node = document()
            .create_element_ns(Some(intern(&namespace)), intern(&tag))
            .unwrap_throw()
            .into();
        DomNode {
            id: Default::default(),
            node,
        }
    }

    /// For performance reasons, we will render this template to an HTML string and then cache it.
    ///
    /// We can then cerate an HTML template element and clone it to create a new instance.
    fn instantiate_template(template: &Template) -> TemplateResult<DomNode> {
        if let Some(cached) = try_get_cached_template(template.id) {
            let root = cached.clone_template_content();

            // Execute the walk sequence.
            let WalkResult {
                flagged_nodes,
                dyn_markers,
            } = execute_walk(&cached.walk, &root, false);

            TemplateResult {
                root: DomNode::from_web_sys(root),
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

impl Html for DomNode {
    const IS_BROWSER: bool = true;

    fn to_web_sys(&self) -> web_sys::Node {
        self.node.clone()
    }

    fn from_web_sys(node: Node) -> Self {
        Self {
            id: Default::default(),
            node,
        }
    }
}

/// Render a [`View`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render(view: impl FnOnce() -> View<DomNode> + 'static) {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    render_to(view, &document.body().unwrap_throw());
}

/// Render a [`View`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render_to(view: impl FnOnce() -> View<DomNode> + 'static, parent: &Node) {
    // Do not call the destructor function, effectively leaking the scope.
    let _ = create_root(|| render_in_scope(view, parent));
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
/// It is expected that this function will be called inside a reactive root, usually created using
/// [`create_root`].
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render_in_scope(view: impl FnOnce() -> View<DomNode> + 'static, parent: &Node) {
    insert(
        &DomNode::from_web_sys(parent.clone()),
        view(),
        None,
        None,
        false,
    );
}

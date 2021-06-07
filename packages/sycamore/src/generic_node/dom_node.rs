//! Rendering backend for the DOM.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Comment, Element, Event, Node, Text};

use crate::generic_node::render::insert;
use crate::generic_node::{EventListener, GenericNode};
use crate::reactive::{create_root, ReactiveScope};
use crate::template_result::TemplateResult;

// TODO: remove js snippet
#[wasm_bindgen(inline_js = "\
export function set_node_id(node, id) {\
    node.__sycamoreNodeId = id\
}\
export function get_node_id(node) {\
    return node.__sycamoreNodeId\
}\
")]
extern "C" {
    fn set_node_id(node: &Node, id: usize);
    fn get_node_id(node: &Node) -> Option<usize>;
}

/// An unique id for every node.
#[derive(Clone, PartialEq, Eq, Hash)]
struct NodeId(usize);

impl NodeId {
    fn new_with_node(node: &Node) -> Self {
        thread_local!(static NODE_ID_COUNTER: Cell<usize> = Cell::new(0));

        let id = NODE_ID_COUNTER.with(|x| {
            let tmp = x.get();
            x.set(tmp + 1);
            tmp
        });
        set_node_id(node, id);
        Self(id)
    }
}

/// Rendering backend for the DOM.
///
/// _This API requires the following crate features to be activated: `dom`_
#[derive(Clone)]
pub struct DomNode {
    id: NodeId,
    node: Rc<Node>,
}

impl DomNode {
    pub fn inner_element(&self) -> Node {
        (*self.node).clone()
    }

    pub fn unchecked_into<T: JsCast>(self) -> T {
        (*self.node).clone().unchecked_into()
    }
}

impl PartialEq for DomNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DomNode {}

impl Hash for DomNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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
            self.node.to_string().as_string().unwrap()
        };
        f.debug_tuple("DomNode").field(&outer_html).finish()
    }
}

fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

impl GenericNode for DomNode {
    fn element(tag: &str) -> Self {
        let node = Rc::new(document().create_element(tag).unwrap().dyn_into().unwrap());
        DomNode {
            id: NodeId::new_with_node(&node),
            node,
        }
    }

    fn text_node(text: &str) -> Self {
        let node = Rc::new(document().create_text_node(text).into());
        DomNode {
            id: NodeId::new_with_node(&node),
            node,
        }
    }

    fn marker() -> Self {
        let node = Rc::new(document().create_comment("").into());
        DomNode {
            id: NodeId::new_with_node(&node),
            node,
        }
    }

    fn set_attribute(&self, name: &str, value: &str) {
        self.node
            .unchecked_ref::<Element>()
            .set_attribute(name, value)
            .unwrap();
    }

    fn set_property(&self, name: &str, value: &JsValue) {
        assert!(js_sys::Reflect::set(&self.node, &name.into(), value).unwrap());
    }

    fn append_child(&self, child: &Self) {
        self.node.append_child(&child.node).unwrap();
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        self.node
            .insert_before(&new_node.node, reference_node.map(|n| n.node.as_ref()))
            .unwrap();
    }

    fn remove_child(&self, child: &Self) {
        self.node.remove_child(&child.node).unwrap();
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        self.node.replace_child(&new.node, &old.node).unwrap();
    }

    fn insert_sibling_before(&self, child: &Self) {
        self.node
            .unchecked_ref::<Element>()
            .before_with_node_1(&child.node)
            .unwrap();
    }

    fn parent_node(&self) -> Option<Self> {
        self.node.parent_node().map(|node| Self {
            id: NodeId(get_node_id(&node).unwrap()),
            node: Rc::new(node),
        })
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self {
            id: NodeId(get_node_id(&node).unwrap()),
            node: Rc::new(node),
        })
    }

    fn remove_self(&self) {
        self.node.unchecked_ref::<Element>().remove();
    }

    fn event(&self, name: &str, handler: Box<EventListener>) {
        type EventListener = dyn Fn(Event);

        thread_local! {
            /// A global event listener pool to prevent [`Closure`]s from being deallocated.
            /// TODO: remove events when elements are detached.
            static EVENT_LISTENERS: RefCell<Vec<Closure<EventListener>>> = RefCell::new(Vec::new());
        }

        let closure = Closure::wrap(handler);
        self.node
            .add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())
            .unwrap();

        EVENT_LISTENERS.with(|event_listeners| event_listeners.borrow_mut().push(closure));
    }

    fn update_inner_text(&self, text: &str) {
        self.node.set_text_content(Some(text));
    }
}

/// Render a [`TemplateResult`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render(template_result: impl FnOnce() -> TemplateResult<DomNode>) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    render_to(template_result, &document.body().unwrap());
}

/// Render a [`TemplateResult`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render_to(template_result: impl FnOnce() -> TemplateResult<DomNode>, parent: &Node) {
    let scope = create_root(|| {
        insert(
            DomNode {
                id: NodeId::new_with_node(parent),
                node: Rc::new(parent.clone()),
            },
            template_result(),
            None,
            None,
        );
    });

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));
}

/// Render a [`TemplateResult`] under a `parent` node by reusing existing nodes (client side
/// hydration). Alias for [`hydrate_to`] with `parent` being the `<body>` tag.
///
/// For rendering without hydration, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn hydrate(template_result: impl FnOnce() -> TemplateResult<DomNode>) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    hydrate_to(template_result, &document.body().unwrap());
}

/// Gets the children of an [`Element`] by collecting them into a [`Vec`]. Note that the returned
/// value is **NOT** live.
fn get_children(parent: &Element) -> Vec<Element> {
    let children = parent.children();
    let children_count = children.length();

    let mut vec = Vec::new();
    vec.reserve(children_count as usize);

    for i in 0..children.length() {
        vec.push(children.get_with_index(i).unwrap());
    }

    vec
}

/// Render a [`TemplateResult`] under a `parent` node by reusing existing nodes (client side
/// hydration). For rendering under the `<body>` tag, use [`hydrate_to`] instead.
///
/// For rendering without hydration, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn hydrate_to(template_result: impl FnOnce() -> TemplateResult<DomNode>, parent: &Node) {
    for child in get_children(parent.unchecked_ref()) {
        child.remove();
    }

    let scope = create_root(|| {
        insert(
            DomNode {
                id: NodeId::new_with_node(&parent),
                node: Rc::new(parent.clone()),
            },
            template_result(),
            None,
            None, // TODO
        );
    });

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));
}

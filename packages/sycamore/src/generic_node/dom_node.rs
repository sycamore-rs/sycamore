//! Rendering backend for the DOM.

use std::cell::Cell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{intern, JsCast};
use web_sys::{Comment, Element, Node, Text};

use crate::utils::render::insert;
use crate::generic_node::{EventHandler, GenericNode};
use crate::rx::{create_root, on_cleanup, ReactiveScope};
use crate::template::Template;

// TODO: remove js snippet
#[wasm_bindgen(inline_js = "\
export function set_node_id(node, id) {\
    node.$$$nodeId = id\
}\
export function get_node_id(node) {\
    return node.$$$nodeId\
}\
")]
extern "C" {
    fn set_node_id(node: &Node, id: usize);
    fn get_node_id(node: &Node) -> Option<usize>;
}

/// An unique id for every node.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId(usize);

impl Default for NodeId {
    fn default() -> Self {
        Self(0)
    }
}

impl NodeId {
    fn new_with_node(node: &Node) -> Self {
        thread_local!(static NODE_ID_COUNTER: Cell<usize> = Cell::new(1)); // 0 is reserved for default value.

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
    id: Cell<NodeId>,
    node: Rc<Node>,
}

impl DomNode {
    pub fn inner_element(&self) -> Node {
        (*self.node).clone()
    }

    pub fn unchecked_into<T: JsCast>(self) -> T {
        (*self.node).clone().unchecked_into()
    }

    fn get_node_id(&self) -> NodeId {
        if self.id.get().0 == 0 {
            // self.id not yet initialized.
            if let Some(id) = get_node_id(&self.node) {
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
            self.node.to_string().as_string().unwrap()
        };
        f.debug_tuple("DomNode").field(&outer_html).finish()
    }
}

fn document() -> web_sys::Document {
    thread_local! {
        /// Cache document since it is frequently accessed to prevent going through js-interop.
        static DOCUMENT: web_sys::Document = web_sys::window().unwrap().document().unwrap();
    };
    DOCUMENT.with(|document| document.clone())
}

impl GenericNode for DomNode {
    fn element(tag: &str) -> Self {
        let node = Rc::new(
            document()
                .create_element(intern(tag))
                .unwrap()
                .dyn_into()
                .unwrap(),
        );
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn text_node(text: &str) -> Self {
        let node = Rc::new(document().create_text_node(text).into());
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn marker() -> Self {
        let node = Rc::new(document().create_comment("").into());
        DomNode {
            id: Default::default(),
            node,
        }
    }

    fn set_attribute(&self, name: &str, value: &str) {
        self.node
            .unchecked_ref::<Element>()
            .set_attribute(intern(name), value)
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
            id: Default::default(),
            node: Rc::new(node),
        })
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self {
            id: Default::default(),
            node: Rc::new(node),
        })
    }

    fn remove_self(&self) {
        self.node.unchecked_ref::<Element>().remove();
    }

    fn event(&self, name: &str, handler: Box<EventHandler>) {
        let closure = Closure::wrap(handler);
        self.node
            .add_event_listener_with_callback(intern(name), closure.as_ref().unchecked_ref())
            .unwrap();

        on_cleanup(move || {
            drop(closure);
        });
    }

    fn update_inner_text(&self, text: &str) {
        self.node.set_text_content(Some(text));
    }
}

/// Render a [`Template`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render(template: impl FnOnce() -> Template<DomNode>) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    render_to(template, &document.body().unwrap());
}

/// Render a [`Template`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render_to(template: impl FnOnce() -> Template<DomNode>, parent: &Node) {
    let scope = create_root(|| {
        insert(
            &DomNode {
                id: Default::default(),
                node: Rc::new(parent.clone()),
            },
            template(),
            None,
            None,
        );
    });

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));
}

/// Render a [`Template`] under a `parent` node by reusing existing nodes (client side
/// hydration). Alias for [`hydrate_to`] with `parent` being the `<body>` tag.
///
/// For rendering without hydration, use [`render`] instead.
///
/// **TODO**: This method currently deletes existing nodes from DOM and reinserts new
/// created nodes. This will be fixed in a later release.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn hydrate(template: impl FnOnce() -> Template<DomNode>) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    hydrate_to(template, &document.body().unwrap());
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

/// Render a [`Template`] under a `parent` node by reusing existing nodes (client side
/// hydration). For rendering under the `<body>` tag, use [`hydrate_to`] instead.
///
/// For rendering without hydration, use [`render`] instead.
///
/// **TODO**: This method currently deletes existing nodes from DOM and reinserts new
/// created nodes. This will be fixed in a later release.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn hydrate_to(template: impl FnOnce() -> Template<DomNode>, parent: &Node) {
    for child in get_children(parent.unchecked_ref()) {
        child.remove();
    }

    let scope = create_root(|| {
        insert(
            &DomNode {
                id: Default::default(),
                node: Rc::new(parent.clone()),
            },
            template(),
            None,
            None, // TODO
        );
    });

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));
}

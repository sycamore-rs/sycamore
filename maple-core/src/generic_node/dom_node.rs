use std::cell::RefCell;

use ref_cast::RefCast;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Element, Event, Node, Text};

use crate::generic_node::{EventListener, GenericNode};

#[derive(Debug, Clone, PartialEq, Eq, RefCast)]
#[repr(transparent)]
pub struct DomNode {
    node: Node,
}

impl DomNode {
    pub fn inner_element(&self) -> Node {
        self.node.clone()
    }
    pub fn unchecked_into<T: JsCast>(self) -> T {
        self.node.unchecked_into()
    }
}

impl AsRef<JsValue> for DomNode {
    fn as_ref(&self) -> &JsValue {
        self.node.as_ref()
    }
}

impl From<DomNode> for JsValue {
    fn from(node: DomNode) -> Self {
        node.node.into()
    }
}

impl JsCast for DomNode {
    fn instanceof(val: &JsValue) -> bool {
        Node::instanceof(val)
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        DomNode {
            node: Node::unchecked_from_js(val),
        }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        DomNode::ref_cast(Node::unchecked_from_js_ref(val))
    }
}

fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

impl GenericNode for DomNode {
    fn element(tag: &str) -> Self {
        DomNode {
            node: document().create_element(tag).unwrap().dyn_into().unwrap(),
        }
    }

    fn text_node(text: &str) -> Self {
        DomNode {
            node: document().create_text_node(text).into(),
        }
    }

    fn fragment() -> Self {
        DomNode {
            node: document().create_document_fragment().into(),
        }
    }

    fn empty() -> Self {
        DomNode {
            node: document().create_comment("").into(),
        }
    }

    fn set_attribute(&self, name: &str, value: &str) {
        self.node
            .unchecked_ref::<Element>()
            .set_attribute(name, value)
            .unwrap();
    }

    fn append_child(&self, child: &Self) {
        self.node.append_child(&child.node).unwrap();
    }

    fn insert_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        self.node
            .insert_before(&new_node.node, reference_node.map(|n| &n.node))
            .unwrap();
    }

    fn remove_child(&self, child: &Self) {
        self.node.remove_child(&child.node).unwrap();
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        self.node.replace_child(&old.node, &new.node).unwrap();
    }

    fn insert_sibling_before(&self, child: &Self) {
        self.node
            .unchecked_ref::<Element>()
            .before_with_node_1(&child.node)
            .unwrap();
    }

    fn parent_node(&self) -> Option<Self> {
        self.node.parent_node().map(|node| Self { node })
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self { node })
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

    fn update_text(&self, text: &str) {
        self.node
            .dyn_ref::<Text>()
            .unwrap()
            .set_text_content(Some(text));
    }
}

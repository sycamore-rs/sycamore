//! Internal DOM manipulation utilities. Generated by the `template!` macro. Should not be used directly.
//! Internal APIs can be changed at any time without a major release.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DocumentFragment, Element, Event, Node};

use crate::prelude::*;

/// Create a new [`Element`] with the specified tag.
pub fn element(tag: &str) -> Element {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element(tag)
        .unwrap()
        .dyn_into()
        .unwrap()
}

pub fn fragment() -> DocumentFragment {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_document_fragment()
}

/// Create a new [`Node`] with the specified text content.
pub fn text(value: impl Fn() -> String + 'static) -> Node {
    let text_node = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_text_node("" /* placeholder */);

    create_effect({
        let text_node = text_node.clone();
        move || {
            text_node.set_text_content(Some(&value()));
        }
    });

    text_node.into()
}

/// Sets an attribute on an [`Element`].
pub fn attr(element: &Element, name: &str, value: impl Fn() -> String + 'static) {
    let element = element.clone();
    let name = name.to_string();
    create_effect(move || {
        element.set_attribute(&name, &value()).unwrap();
    })
}

type EventListener = dyn Fn(Event);

thread_local! {
    /// A global event listener pool to prevent [`Closure`]s from being deallocated.
    /// TODO: remove events when elements are detached.
    static EVENT_LISTENERS: RefCell<Vec<Closure<EventListener>>> = RefCell::new(Vec::new());
}

/// Sets an event listener on an [`Element`].
pub fn event(element: &Element, name: &str, handler: Box<EventListener>) {
    let closure = Closure::wrap(handler);
    element
        .add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())
        .unwrap();

    EVENT_LISTENERS.with(|event_listeners| event_listeners.borrow_mut().push(closure));
}

/// Appends a child node to an element.
pub fn append(element: &impl AsRef<Node>, child: &impl AsRef<Node>) {
    element.as_ref().append_child(child.as_ref()).unwrap();
}

/// Appends a [`dyn Render`](Render) to the `parent` node.
/// Node is created inside an effect with [`Render::update_node`].
pub fn append_render(parent: &impl AsRef<Node>, child: Box<dyn Fn() -> Box<dyn Render>>) {
    let parent = parent.as_ref().clone();

    let node = create_effect_initial(cloned!((parent) => Box::new(move || {
        let node = RefCell::new(child().render());

        let effect = cloned!((node) => move || {
            let new_node = child().update_node(&parent, &node.borrow());
            *node.borrow_mut() = new_node;
        });

        (Rc::new(effect), node)
    })));

    parent.append_child(&node.borrow()).unwrap();
}

/// Sets the value of a [`NodeRef`].
pub fn set_noderef(node: &impl AsRef<Node>, noderef: NodeRef) {
    noderef.set(node.as_ref().clone());
}

use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, Node, Text};

/// Create a new [`HtmlElement`] with the specified tag.
pub fn element(tag: &str) -> HtmlElement {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element(tag)
        .unwrap()
        .dyn_into()
        .unwrap()
}

pub fn text(value: &str) -> Text {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_text_node(value)
}

/// Sets an attribute on an [`HtmlElement`].
pub fn attr(element: &HtmlElement, name: &str, value: &str) {
    element.set_attribute(name, value).unwrap();
}

/// Appends a child node to an element.
pub fn append(element: &Element, child: &Node) {
    element.append_with_node_1(&child).unwrap();
}

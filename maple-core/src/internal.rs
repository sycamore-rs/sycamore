use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

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

/// Sets an attribute on an [`HtmlElement`].
pub fn attr(element: &HtmlElement, name: &str, value: &str) {
    element.set_attribute(name, value).unwrap();
}

pub fn append(element: &HtmlElement, child: HtmlElement) {
    element.append_with_node_1(&child).unwrap();
}

use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement, Window};

pub(crate) fn window() -> Window {
    web_sys::window().unwrap()
}

pub(crate) fn document() -> Document {
    window().document().unwrap()
}

pub(crate) fn query(selectors: &str) -> Element {
    document()
        .query_selector(selectors)
        .expect("selectors should be valid")
        .expect("element to be found that matches the selectors")
}

pub(crate) fn query_into<T: AsRef<HtmlElement> + JsCast>(selectors: &str) -> T {
    // dyn_into -> unwrap to eagerly cause a panic if the query doesn't match
    // the generic T.
    query(selectors)
        .dyn_into()
        .expect("element found should be of the same type as used for the generic T")
}

macro_rules! assert_text_content {
    ($element: expr, $right: expr $(,)?) => {
        assert_eq!($element.text_content().unwrap(), $right);
    };
}

pub(crate) use assert_text_content;

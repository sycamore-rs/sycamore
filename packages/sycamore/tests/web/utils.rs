use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

/// Query the `Document` for the first `Element` that matches the selectors.
///
/// This is a test utility function only!
///
/// # Panics
///
/// Panics if the selectors string is invalid or if no element was found that
/// matches the selectors
pub(crate) fn query(selectors: &str) -> Element {
    document()
        .query_selector(selectors)
        .expect("selectors should be valid")
        .expect("element to be found that matches the selectors")
}

/// Query the `Document` for the first `Element` that matches the selectors and
/// then try to cast it into the generic type `T`.
///
/// This is a test utility function only!
///
/// # Panics
///
/// Panics if:
/// - the selectors string is invalid
/// - no element was found that matches the selectors
/// - element found cannot be cast to the generic type `T` used
pub(crate) fn query_into<T: AsRef<HtmlElement> + JsCast>(selectors: &str) -> T {
    // dyn_into -> unwrap to eagerly cause a panic if the query doesn't match
    // the generic T.
    query(selectors)
        .dyn_into()
        .expect("element found should be of the same type as used for the generic T")
}

/// Asserts that the text content of a `web_sys::Node` is equal to the
/// right expression.
macro_rules! assert_text_content {
    ($element: expr, $right: expr $(,)?) => {
        assert_eq!($element.text_content().unwrap(), $right);
    };
}

pub(crate) use assert_text_content;

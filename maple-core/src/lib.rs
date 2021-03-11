//! # Maple API Documentation
//!
//! Maple is a VDOM-less web library with fine-grained reactivity.
//!
//! This is the API docs for maple. If you are looking for the usage docs, checkout the [README](https://github.com/lukechu10/maple).
//!
//! ## Supported Targets
//! - `wasm32-unknown-unknown`

#[doc(hidden)]
pub mod internal;
#[doc(hidden)]
pub mod macros;
pub mod reactive;

use web_sys::HtmlElement;

/// The result of the `template!` macro. Should not be used directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateResult {
    element: HtmlElement,
}

/// Render an [`HtmlElement`] into the DOM.
pub fn render(template_result: TemplateResult) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document
        .body()
        .unwrap()
        .append_child(&template_result.element)
        .unwrap();
}

impl TemplateResult {
    /// Create a new `TemplateResult` from an [`HtmlElement`].
    pub fn new(element: HtmlElement) -> Self {
        Self { element }
    }

    pub fn inner_element(&self) -> HtmlElement {
        self.element.clone()
    }
}

/// The maple prelude.
pub mod prelude {
    pub use crate::cloned;
    pub use crate::reactive::{
        create_effect, create_memo, create_selector, create_selector_with, Signal, StateHandle,
    };
    pub use crate::{render, TemplateResult};

    pub use maple_core_macro::template;
}

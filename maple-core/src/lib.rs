//! # Maple API Documentation
//!
//! Maple is a VDOM-less web library with fine-grained reactivity.
//!
//! This is the API docs for maple. If you are looking for the usage docs, checkout the [README](https://github.com/lukechu10/maple).
//!
//! ## Supported Targets
//! - `wasm32-unknown-unknown`

pub mod internal;
pub mod reactive;

use web_sys::HtmlElement;

/// Render an [`HtmlElement`] into the DOM.
pub fn render(element: HtmlElement) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document.body().unwrap().append_child(&element).unwrap();
}

/// The maple prelude.
pub mod prelude {
    pub use crate::reactive::{create_effect, create_memo, create_signal, untracked};
    pub use crate::render;

    pub use maple_core_macro::template;
}

//! # Maple API Documentation
//!
//! Maple is a VDOM-less web library with fine-grained reactivity.
//!
//! This is the API docs for maple. If you are looking for the usage docs, checkout the [README](https://github.com/lukechu10/maple).
//!
//! ## Supported Targets
//! - `wasm32-unknown-unknown`

#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::type_repetition_in_bounds)]

#[doc(hidden)]
pub mod internal;
#[doc(hidden)]
pub mod macros;
pub mod reactive;
pub mod render;

use web_sys::Node;

use std::cell::RefCell;
use std::iter::FromIterator;

/// The result of the `template!` macro. Should not be used directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateResult {
    node: Node,
}

impl TemplateResult {
    /// Create a new `TemplateResult` from a [`Node`].
    pub fn new(node: Node) -> Self {
        Self { node }
    }

    pub fn inner_element(&self) -> Node {
        self.node.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateList {
    templates: Vec<TemplateResult>,
}

impl FromIterator<TemplateResult> for TemplateList {
    fn from_iter<T: IntoIterator<Item = TemplateResult>>(iter: T) -> Self {
        Self {
            templates: FromIterator::from_iter(iter),
        }
    }
}

/// Render a [`TemplateResult`] into the DOM.
pub fn render(template_result: impl FnOnce() -> TemplateResult + 'static) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let owner = reactive::create_root(move || {
        document
            .body()
            .unwrap()
            .append_child(&template_result().node)
            .unwrap();
    });

    thread_local! {
        static GLOBAL_OWNERS: RefCell<Vec<reactive::Owner>> = RefCell::new(Vec::new());
    }

    GLOBAL_OWNERS.with(|global_owners| global_owners.borrow_mut().push(owner));
}

/// The maple prelude.
pub mod prelude {
    pub use crate::cloned;
    pub use crate::reactive::{
        create_effect, create_effect_initial, create_memo, create_root, create_selector,
        create_selector_with, on_cleanup, Signal, StateHandle,
    };
    pub use crate::render::Render;
    pub use crate::{render, TemplateList, TemplateResult};

    pub use maple_core_macro::template;
}

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

use web_sys::Node;

use std::cell::RefCell;
use std::rc::Rc;

/// The result of the `template!` macro. Should not be used directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateResult {
    node: Node,
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
        static GLOBAL_OWNERS: RefCell<Vec<Rc<RefCell<reactive::Owner>>>> = RefCell::new(Vec::new());
    }

    GLOBAL_OWNERS.with(|global_owners| global_owners.borrow_mut().push(owner));
}

impl TemplateResult {
    /// Create a new `TemplateResult` from an [`HtmlElement`].
    pub fn new(node: Node) -> Self {
        Self { node }
    }

    pub fn inner_element(&self) -> Node {
        self.node.clone()
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

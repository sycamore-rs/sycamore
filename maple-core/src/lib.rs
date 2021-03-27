//! # Maple API Documentation
//!
//! Maple is a VDOM-less web library with fine-grained reactivity.
//!
//! This is the API docs for maple. If you are looking for the usage docs, checkout the [README](https://github.com/lukechu10/maple).
//!
//! ## Supported Targets
//! - `wasm32-unknown-unknown`
//!
//! ## Features
//! - `serde` - Enables serializing and deserializing `Signal`s and other wrapper types using `serde`.

#![allow(non_snake_case)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::type_repetition_in_bounds)]

use std::cell::RefCell;

use web_sys::Node;

pub use maple_core_macro::template;
use prelude::SignalVec;

use crate::generic_node::{DomNode, GenericNode};

pub mod flow;
pub mod generic_node;
#[doc(hidden)]
pub mod internal;
#[doc(hidden)]
pub mod macros;
pub mod noderef;
pub mod reactive;
pub mod render;

/// The result of the [`template!`](template) macro. Should not be used directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateResult<G: GenericNode> {
    node: G,
}

impl<G: GenericNode> TemplateResult<G> {
    /// Create a new [`TemplateResult`] from a [`Node`].
    pub fn new(node: G) -> Self {
        Self { node }
    }

    /// Create a new [`TemplateResult`] with a blank comment node
    pub fn empty() -> Self {
        Self::new(G::marker())
    }

    pub fn inner_element(&self) -> G {
        self.node.clone()
    }
}

/// A [`SignalVec`](reactive::SignalVec) of [`TemplateResult`]s.
#[derive(Clone)]
pub struct TemplateList<T: GenericNode> {
    templates: reactive::SignalVec<TemplateResult<T>>,
}

impl<T: GenericNode> From<SignalVec<TemplateResult<T>>> for TemplateList<T> {
    fn from(templates: SignalVec<TemplateResult<T>>) -> Self {
        Self { templates }
    }
}

/// Render a [`TemplateResult`] into the DOM.
///
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
pub fn render(template_result: impl FnOnce() -> TemplateResult<DomNode> + 'static) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    render_to(template_result, &document.body().unwrap());
}

/// Render a [`TemplateResult`] under a `parent` node. For rendering under the `<body>` tag, use [`render()`] instead.
pub fn render_to(
    template_result: impl FnOnce() -> TemplateResult<DomNode> + 'static,
    parent: &Node,
) {
    let owner = reactive::create_root(move || {
        parent
            .append_child(&template_result().node.inner_element())
            .unwrap();
    });

    thread_local! {
        static GLOBAL_OWNERS: RefCell<Vec<reactive::Owner>> = RefCell::new(Vec::new());
    }

    GLOBAL_OWNERS.with(|global_owners| global_owners.borrow_mut().push(owner));
}

/// The maple prelude.
pub mod prelude {
    pub use maple_core_macro::template;

    pub use crate::cloned;
    pub use crate::flow::{Indexed, IndexedProps, Keyed, KeyedProps};
    pub use crate::generic_node::GenericNode;
    pub use crate::noderef::NodeRef;
    pub use crate::reactive::{
        create_effect, create_effect_initial, create_memo, create_root, create_selector,
        create_selector_with, on_cleanup, untrack, Signal, SignalVec, StateHandle,
    };
    pub use crate::render::Render;
    pub use crate::{render, render_to, TemplateList, TemplateResult};
}

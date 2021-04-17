//! # Maple API Documentation
//!
//! Maple is a VDOM-less web library with fine-grained reactivity.
//!
//! This is the API docs for maple. If you are looking for the usage docs, checkout the [README](https://github.com/lukechu10/maple).
//!
//! ## Features
//! - `dom` (_default_) - Enables rendering templates to DOM nodes. Only useful on
//!   `wasm32-unknown-unknown` target.
//! - `ssr` - Enables rendering templates to static strings (useful for Server Side Rendering /
//!   Pre-rendering).
//! - `serde` - Enables serializing and deserializing `Signal`s and other wrapper types using
//!   `serde`.

#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::rc_buffer)]
#![warn(rust_2018_idioms)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::type_repetition_in_bounds)]

pub use maple_core_macro::{component, template};

pub mod component;
pub mod easing;
pub mod flow;
pub mod generic_node;
pub mod macros;
pub mod noderef;
pub mod reactive;
pub mod render;
pub mod template_result;
pub mod utils;

/// Alias self to maple_core for proc-macros.
extern crate self as maple_core;

/// Render a [`TemplateResult`](template_result::TemplateResult) into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
///
/// _This API requires the following crate features to be activated: `dom`_
#[cfg(feature = "dom")]
pub fn render(
    template_result: impl FnOnce() -> template_result::TemplateResult<generic_node::DomNode>,
) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    render_to(template_result, &document.body().unwrap());
}

/// Render a [`TemplateResult`](template_result::TemplateResult) under a `parent` node.
/// For rendering under the `<body>` tag, use [`render()`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
#[cfg(feature = "dom")]
pub fn render_to(
    template_result: impl FnOnce() -> template_result::TemplateResult<generic_node::DomNode>,
    parent: &web_sys::Node,
) {
    let scope = reactive::create_root(|| {
        for node in template_result() {
            parent.append_child(&node.inner_element()).unwrap();
        }
    });

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<reactive::ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));
}

/// Render a [`TemplateResult`](template_result::TemplateResult) into a static [`String`]. Useful
/// for rendering to a string on the server side.
///
/// _This API requires the following crate features to be activated: `ssr`_
#[cfg(feature = "ssr")]
pub fn render_to_string(
    template_result: impl FnOnce() -> template_result::TemplateResult<generic_node::SsrNode>,
) -> String {
    let mut ret = String::new();
    let _scope = reactive::create_root(|| {
        for node in template_result() {
            ret.push_str(&format!("{}", node));
        }
    });

    ret
}

/// The maple prelude.
pub mod prelude {
    pub use maple_core_macro::{component, template};

    pub use crate::cloned;
    pub use crate::flow::{Indexed, IndexedProps, Keyed, KeyedProps};
    #[cfg(feature = "dom")]
    pub use crate::generic_node::DomNode;
    pub use crate::generic_node::GenericNode;
    #[cfg(feature = "ssr")]
    pub use crate::generic_node::SsrNode;
    pub use crate::noderef::NodeRef;
    pub use crate::reactive::{
        create_effect, create_effect_initial, create_memo, create_root, create_selector,
        create_selector_with, on_cleanup, untrack, Signal, StateHandle,
    };
    pub use crate::render::Render;
    #[cfg(feature = "ssr")]
    pub use crate::render_to_string;
    pub use crate::template_result::TemplateResult;
    #[cfg(feature = "dom")]
    pub use crate::{render, render_to};
}

/// Re-exports for use by `maple-core-macro`. Not intended for use by end-users.
#[doc(hidden)]
pub mod rt {
    pub use js_sys::Reflect;
    pub use wasm_bindgen::{JsCast, JsValue};
    pub use web_sys::Event;
}

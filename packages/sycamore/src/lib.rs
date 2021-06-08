//! # Sycamore API Documentation
//!
//! Sycamore is a VDOM-less web library with fine-grained reactivity.
//!
//! This is the API docs for sycamore. If you are looking for the usage docs, checkout the
//! [Sycamore Book](https://sycamore-rs.netlify.app/getting_started/installation).
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
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(rust_2018_idioms)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::type_repetition_in_bounds)]

pub use sycamore_macro::{component, template};

pub mod component;
pub mod easing;
pub mod flow;
pub mod generic_node;
pub mod macros;
pub mod noderef;
pub mod render;
pub mod rx;
pub mod template;
pub mod utils;

/// Alias self to sycamore for proc-macros.
extern crate self as sycamore;

/// The sycamore prelude.
pub mod prelude {
    pub use sycamore_macro::{component, template};

    pub use crate::cloned;
    pub use crate::flow::{Indexed, IndexedProps, Keyed, KeyedProps};
    pub use crate::generic_node::GenericNode;
    #[cfg(feature = "dom")]
    pub use crate::generic_node::{hydrate, hydrate_to, render, render_to, DomNode};
    #[cfg(feature = "ssr")]
    pub use crate::generic_node::{render_to_string, SsrNode};
    pub use crate::noderef::NodeRef;
    pub use crate::render::IntoTemplate;
    pub use crate::render::IntoTemplate;
    pub use crate::rx::{
        create_effect, create_effect_initial, create_memo, create_root, create_selector,
        create_selector_with, on_cleanup, untrack, Signal, StateHandle,
    };
    pub use crate::template::Template;
}

/// Re-exports for use by `sycamore-macro`. Not intended for use by end-users.
#[doc(hidden)]
pub mod rt {
    pub use js_sys::Reflect;
    pub use wasm_bindgen::{JsCast, JsValue};
    pub use web_sys::Event;
}

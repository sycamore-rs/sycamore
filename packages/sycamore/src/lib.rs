//! # Sycamore API Documentation
//!
//! Sycamore is a VDOM-less web library with fine-grained reactivity.
//!
//! This is the API docs for sycamore. If you are looking for the usage docs, checkout the
//! [Sycamore Book](https://sycamore-rs.netlify.app/docs/getting_started/installation).
//!
//! ## Feature Flags
//! - `dom` (_default_) - Enables rendering templates to DOM nodes. Only useful on
//!   `wasm32-unknown-unknown` target.
//! - `experimental-builder-agnostic` - Enables the agnostic backend builder API.
//! - `experimental-builder-html` - Enables the HTML specific backend builder API. Also enables
//!   `experimental-builder-agnostic`.
//! - `hydrate` - Enables client-side hydration support.
//! - `futures` - Enables wrappers around `wasm-bindgen-futures` to make it easier to extend a
//!   reactive scope into an `async` function.
//! - `ssr` - Enables rendering templates to static strings (useful for Server Side Rendering /
//!   Pre-rendering).
//! - `serde` - Enables serializing and deserializing `Signal`s and other wrapper types using
//!   `serde`.
//! - `wasm-bindgen-interning` (_default_) - Enables interning for `wasm-bindgen` strings. This
//!   improves performance at a slight cost in binary size. If you want to minimize the size of the
//!   result `.wasm` binary, you might want to disable this.

#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::rc_buffer)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(rust_2018_idioms)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::type_repetition_in_bounds)]

pub use sycamore_macro::{component, node, view};
pub use sycamore_reactive as reactive;

pub mod component;
pub mod context;
pub mod easing;
pub mod flow;
pub mod generic_node;
pub mod motion;
pub mod noderef;
pub mod portal;
pub mod utils;
pub mod view;

#[cfg(feature = "experimental-builder-agnostic")]
pub mod builder;

#[cfg(feature = "hydrate")]
pub mod hydrate;

#[cfg(feature = "futures")]
pub mod futures;

/// Alias self to sycamore for proc-macros.
extern crate self as sycamore;

#[cfg(feature = "dom")]
pub use crate::generic_node::{hydrate, hydrate_to, render, render_to, DomNode};
#[cfg(feature = "ssr")]
pub use crate::generic_node::{render_to_string, SsrNode};

/// The sycamore prelude.
pub mod prelude {
    pub use sycamore_macro::{component, node, view};

    #[cfg(feature = "experimental-builder-agnostic")]
    pub use crate::builder::agnostic::prelude::*;
    pub use crate::flow::{Indexed, IndexedProps, Keyed, KeyedProps};
    #[cfg(feature = "dom")]
    pub use crate::generic_node::DomNode;
    pub use crate::generic_node::GenericNode;
    pub use crate::generic_node::Html;
    #[cfg(feature = "ssr")]
    pub use crate::generic_node::SsrNode;
    pub use crate::noderef::NodeRef;
    pub use crate::reactive::{
        cloned, create_effect, create_memo, create_root, create_selector, create_selector_with,
        on_cleanup, untrack, Signal, StateHandle,
    };
    pub use crate::view::{IntoView, View};
}

/// Re-exports for use by `sycamore-macro`. Not intended for use by end-users.
#[doc(hidden)]
pub mod rt {
    pub use js_sys::Reflect;
    pub use wasm_bindgen::{intern, JsCast, JsValue};
    pub use web_sys::Event;
}

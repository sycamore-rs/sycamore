//! # Sycamore API Documentation
//!
//! Sycamore is a VDOM-less web library with fine-grained reactivity.
//!
//! This is the API docs for sycamore. If you are looking for the usage docs, checkout the
//! [Sycamore Book](https://sycamore-rs.netlify.app/docs/getting_started/installation).
//!
//! ## Feature Flags
//!
//! - `hydrate` - Enables client-side hydration support.
//!
//! - `suspense` - Enables wrappers around `wasm-bindgen-futures` to make it easier to extend a
//!   reactive scope into an `async` function.
//!
//! - `ssr` - Enables rendering templates to static strings (useful for Server Side Rendering /
//!   Pre-rendering).
//!
//! - `serde` - Enables serializing and deserializing `Signal`s and other wrapper types using
//!   `serde`.
//!
//! - `wasm-bindgen-interning` (_default_) - Enables interning for `wasm-bindgen` strings. This
//!   improves performance at a slight cost in binary size. If you want to minimize the size of the
//!   result `.wasm` binary, you might want to disable this.
//!
//! - `web` (_default_) - Enables the web backend for Sycamore. This feature is enabled by most of
//!   the other features so you should rarely need to enable it manually.

#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::rc_buffer)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::type_repetition_in_bounds)]
#![deny(missing_debug_implementations)]

// Alias self to `sycamore` to make it possible to use proc-macros within the `sycamore` crate.
#[allow(unused_extern_crates)] // False positive
extern crate self as sycamore;

// pub mod builder;
pub mod easing;
pub mod flow;
#[cfg(feature = "suspense")]
pub mod futures;
pub mod motion;
#[cfg(feature = "suspense")]
pub mod suspense;
#[cfg(feature = "web")]
pub mod web;

/* Re-export modules from sycamore-core */
pub use sycamore_core::{component, generic_node, noderef, view};
/* Re-export of the sycamore-macro crate */
pub use sycamore_macro::*;

/// Re-export of the `sycamore-reactive` crate.
///
/// Reactive primitives for Sycamore.
pub mod reactive {
    pub use sycamore_reactive::*;
}

#[cfg(all(feature = "web", feature = "hydrate"))]
pub use web::render::{hydrate, hydrate_to, hydrate_to_with_scope};
#[cfg(feature = "web")]
pub use web::render::{render, render_to, render_to_with_scope};
#[cfg(feature = "ssr")]
pub use web::render::{render_to_string, render_to_string_with_scope};
// #[cfg(all(feature = "ssr", feature = "suspense"))]
// pub use web::render_to_string_await_suspense; // FIXME

/// The sycamore prelude.
///
/// In most cases, it is idiomatic to use a glob import (aka wildcard import) at the beginning of
/// your Rust source file.
///
/// ```rust
/// use sycamore::prelude::*;
/// ```
pub mod prelude {
    pub use sycamore_macro::*;

    pub use crate::component::{AttributeValue, Attributes, Children};
    pub use crate::flow::*;
    pub use crate::generic_node::GenericNode;
    pub use crate::noderef::{create_node_ref, NodeRef};
    pub use crate::reactive::*;
    #[cfg(not(feature = "web"))]
    pub use crate::view::View;
    #[cfg(feature = "web")]
    pub use crate::web::macros::{node, view};
    #[cfg(feature = "web")]
    pub use crate::web::web_node::WebNode;
    #[cfg(feature = "web")]
    pub use crate::web::{on_mount, View};
}

/// Re-exports for use by `sycamore-macro`. Not intended for use by end-users.
#[doc(hidden)]
pub mod rt {
    #[cfg(feature = "web")]
    pub use js_sys::Reflect;
    #[cfg(feature = "web")]
    pub use wasm_bindgen::{intern, JsCast, JsValue};
    #[cfg(feature = "web")]
    pub use web_sys::Event;
}

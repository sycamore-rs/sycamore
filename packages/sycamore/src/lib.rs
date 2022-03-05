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
//!
//! - `experimental-builder-agnostic` - Enables the agnostic backend builder API.
//!
//! - `experimental-hydrate` - Enables client-side hydration support.
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

#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::rc_buffer)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(rust_2018_idioms)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::type_repetition_in_bounds)]

// Alias self to `sycamore` to make it possible to use proc-macros within the `sycamore` crate.
#[allow(unused_extern_crates)] // False positive
extern crate self as sycamore;

#[cfg(feature = "experimental-builder-agnostic")]
pub mod builder;
pub mod component;
pub mod easing;
pub mod flow;
#[cfg(feature = "suspense")]
pub mod futures;
pub mod generic_node;
pub mod html;
pub mod motion;
pub mod noderef;
pub mod portal;
#[cfg(feature = "suspense")]
pub mod suspense;
pub mod utils;
pub mod view;

/// Re-export for `sycamore-reactive` crate.
///
/// Reactive primitives for Sycamore.
pub mod reactive {
    pub use sycamore_reactive::*;
}

#[cfg(feature = "ssr")]
pub use crate::generic_node::render_to_string;
#[cfg(all(feature = "dom", feature = "experimental-hydrate"))]
pub use generic_node::{hydrate, hydrate_get_scope, hydrate_to};
#[cfg(feature = "dom")]
pub use generic_node::{render, render_get_scope, render_to};

pub use sycamore_macro::*;

/// The sycamore prelude.
///
/// In most cases, it is idiomatic to use a glob import (aka wildcard import) at the beginning of
/// your Rust source file.
///
/// ```rust
/// use sycamore::prelude::*;
/// ```
pub mod prelude {
    #[cfg(feature = "dom")]
    pub use crate::generic_node::DomNode;
    #[cfg(all(feature = "dom", feature = "experimental-hydrate"))]
    pub use crate::generic_node::HydrateNode;
    #[cfg(feature = "ssr")]
    pub use crate::generic_node::SsrNode;

    pub use crate::component::Children;
    pub use crate::flow::*;
    pub use crate::generic_node::{GenericNode, Html};
    pub use crate::noderef::{NodeRef, ScopeCreateNodeRef};
    pub use crate::reactive::*;
    pub use crate::view::View;

    pub use sycamore_macro::*;
}

/// Re-exports for use by `sycamore-macro`. Not intended for use by end-users.
#[doc(hidden)]
pub mod rt {
    pub use js_sys::Reflect;
    pub use wasm_bindgen::{intern, JsCast, JsValue};
    pub use web_sys::Event;
}

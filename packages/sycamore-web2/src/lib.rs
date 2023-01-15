//! # `sycamore-web`
//!
//! This crate adds web support to the Sycamore UI framework. This includes both client-side
//! rendering to the DOM (using `wasm-bindgen` and `web-sys`) and server-side-rendering to render
//! your web app to a static HTML string.

use once_cell::sync::Lazy;
use wasm_bindgen::UnwrapThrowExt;

pub mod hydrate;
pub mod render;
pub mod web_node;

/// Get the global `window` object.
pub fn window() -> web_sys::Window {
    web_sys::window().unwrap_throw()
}

/// Get the global `document` object.
pub fn document() -> web_sys::Document {
    window().document().unwrap_throw()
}

static VOID_ELEMENTS: Lazy<hashbrown::HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr", "command", "keygen", "menuitem",
    ]
    .into_iter()
    .collect()
});

//! # `sycamore-web`
//!
//! This crate adds web support to the Sycamore UI framework. This includes both client-side
//! rendering to the DOM (using `wasm-bindgen` and `web-sys`) and server-side-rendering to render
//! your web app to a static HTML string.

use once_cell::sync::Lazy;
use sycamore_reactive::{Scope, use_scope_status, create_ref};
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};

pub mod html;
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

#[doc(hidden)]
pub static VOID_ELEMENTS: Lazy<hashbrown::HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr", "command", "keygen", "menuitem",
    ]
    .into_iter()
    .collect()
});

/// A `View` type that uses [`WebNode`](web_node::WebNode) as the rendering backend by default.
pub type View<G = web_node::WebNode> = sycamore_core2::view::View<G>;

/// Queue up a callback to be executed when the component is mounted.
///
/// If not on `wasm32` target, does nothing.
///
/// # Potential Pitfalls
///
/// If called inside an async-component, the callback will be called after the next suspension
/// point (when there is an `.await`).
pub fn on_mount<'a>(cx: Scope<'a>, f: impl Fn() + 'a) {
    if cfg!(target_arch = "wasm32") {
        let scope_status = use_scope_status(cx);

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_name = "queueMicrotask")]
            fn queue_microtask(f: &Closure<dyn Fn()>);
        }

        let f: Box<dyn Fn()> = Box::new(f);
        // SAFETY: We do not access `f_extended` until we verify that the scope is still valid using
        // `use_scope_status`.
        let f_extended: Box<dyn Fn() + 'static> = unsafe { std::mem::transmute(f) };

        let cb = move || {
            if *scope_status.get() {
                // Scope is still valid. We can safely execute the callback.
                f_extended();
            }
        };
        let boxed: Box<dyn Fn()> = Box::new(cb);
        let closure = create_ref(cx, Closure::wrap(boxed));
        queue_microtask(closure);
    }
}

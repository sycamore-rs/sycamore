//! # `sycamore-web`
//!
//! This crate adds web support to the Sycamore UI framework. This includes both client-side
//! rendering to the DOM (using `wasm-bindgen` and `web-sys`) and server-side-rendering to render
//! your web app to a static HTML string.
//!
//! # `WebNode`
//!
//! [`WebNode`](crate::web_node::WebNode) is the central part of this crate. This is an
//! implementation of a Sycamore rendering backend for rendering your Sycamore app to HTML, whether
//! it be using the browser's DOM or to a static HTML string.
//!
//! # Feature Flags
//!
//! - `dom`: Enables rendering `WebNode`s to the browser DOM.
//! - `hydrate`: Enables hydration of existing DOM nodes (usually in conjunction with SSR).
//! - `ssr`: Enables rendering `WebNode`s to a static HTML string.
//! - `suspense`: Enables support for futures/suspense integration.
//!
//! - `silence_dom_ssr_features_error`: By default, if both the `dom` and `ssr` features are
//!   enabled, a compile-time error is emitted. This is for code-bloat reasons when deploying to
//!   WASM. If you did intend to enable both features, you can silence this error by enabling this
//!   feature.

use once_cell::sync::Lazy;
use sycamore_reactive::{create_ref, use_scope_status, Scope};
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::UnwrapThrowExt;

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

#[cfg(all(
    not(feature = "silence_dom_ssr_features_error"),
    feature = "dom",
    feature = "ssr"
))]
compile_error!(
    "You should usually only enable either the `dom` feature or the `ssr` feature but not both.
If you did intend to enable both features at the same time, you can enable the `silence_dom_ssr_features_error` feature to silence this error."
);

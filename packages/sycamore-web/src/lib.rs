//! Web renderer for the Sycamore UI framework.
//!
//! Sycamore on its own is a backend-agnostic UI framework. This crate provides web support to
//! Sycamore. With this crate, it is possible to render Sycamore views to the DOM (using
//! [`DomNode`]), "hydrate" existing DOM nodes (using [`HydrateNode`]), or render a static string
//! (using [`SsrNode`]).
//!
//! This crate is re-exported in the `sycamore` crate. It is recommended to use that instead of
//! using this crate directly.

mod dom_node;
#[cfg(feature = "hydrate")]
mod hydrate_node;
#[cfg(feature = "hydrate")]
pub mod hydrate_web;
#[cfg(feature = "ssr")]
mod ssr_node;

pub use dom_node::*;
#[cfg(feature = "hydrate")]
pub use hydrate_node::*;
#[cfg(feature = "hydrate")]
pub use hydrate_web::*;
#[cfg(feature = "ssr")]
pub use ssr_node::*;
use sycamore_core::generic_node::GenericNode;
use sycamore_reactive::*;
use wasm_bindgen::prelude::*;
use web_sys::Event;

/// Trait that is implemented by all [`GenericNode`] backends that render to HTML.
pub trait Html: GenericNode<EventType = Event, PropertyType = JsValue> {
    /// A boolean indicating whether this node is rendered in a browser context.
    ///
    /// A value of `false` does not necessarily mean that it is not being rendered in WASM or even
    /// in the browser. It only means that it does not create DOM nodes.
    const IS_BROWSER: bool;
}

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

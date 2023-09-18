//! Web renderer for the Sycamore UI framework.
//!
//! Sycamore on its own is a backend-agnostic UI framework. This crate provides web support to
//! Sycamore. With this crate, it is possible to render Sycamore views to the DOM (using
//! [`DomNode`]), "hydrate" existing DOM nodes (using [`HydrateNode`]), or render a static string
//! (using [`SsrNode`]).
//!
//! This crate is re-exported in the `sycamore` crate. It is recommended to use that instead of
//! using this crate directly.

#![deny(missing_debug_implementations)]

mod dom_node;
mod dom_node_template;
#[cfg(feature = "hydrate")]
pub mod hydrate;
#[cfg(feature = "hydrate")]
mod hydrate_node;
#[cfg(feature = "ssr")]
mod ssr_node;

use std::cell::Cell;
use std::rc::Rc;

pub use dom_node::*;
#[cfg(feature = "hydrate")]
pub use hydrate_node::*;
use once_cell::sync::Lazy;
#[cfg(feature = "ssr")]
pub use ssr_node::*;
use sycamore_core::generic_node::{GenericNode, GenericNodeElements};
use sycamore_reactive::*;
use wasm_bindgen::prelude::*;

/// Trait that is implemented by all [`GenericNode`] backends that render to HTML.
pub trait Html:
    GenericNode<AnyEventData = JsValue, PropertyType = JsValue> + GenericNodeElements
{
    /// A boolean indicating whether this node is rendered in a browser context.
    ///
    /// A value of `false` does not necessarily mean that it is not being rendered in WASM or even
    /// in the browser. It only means that it does not create DOM nodes.
    const IS_BROWSER: bool;

    /// Convert this node into a raw [`web_sys::Node`].
    ///
    /// For certain backends, this is not possible (e.g. [`SsrNode`]). In that case, calling this
    /// will panic at runtime.
    fn to_web_sys(&self) -> web_sys::Node;

    /// Convert a raw [`web_sys::Node`] into a [`GenericNode`].
    ///
    /// This is the inverse of [`to_web_sys`]. For certain backends, this is not possible (e.g.
    /// [`SsrNode`]). In that case, calling this will panic at runtime.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use sycamore::prelude::*;
    /// # fn get_web_sys_node() -> web_sys::Node {
    /// #     todo!()
    /// # }
    /// # fn my_raw_node_view<G: Html>() -> View<G> {
    /// let raw_node: web_sys::Node = get_web_sys_node();
    /// let node = G::from_web_sys(raw_node);
    /// let view = View::new_node(node);
    /// # view
    /// # }
    /// ```
    fn from_web_sys(node: web_sys::Node) -> Self;
}

static VOID_ELEMENTS: Lazy<hashbrown::HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr", "command", "keygen", "menuitem",
    ]
    .into_iter()
    .collect()
});

/// Queue up a callback to be executed when the component is mounted.
///
/// If not on `wasm32` target, does nothing.
///
/// # Potential Pitfalls
///
/// If called inside an async-component, the callback will be called after the next suspension
/// point (when there is an `.await`).
pub fn on_mount(f: impl Fn() + 'static) {
    if cfg!(target_arch = "wasm32") {
        let is_alive = Rc::new(Cell::new(true));
        on_cleanup({
            let is_alive = Rc::clone(&is_alive);
            move || is_alive.set(false)
        });

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_name = "queueMicrotask")]
            fn queue_microtask(f: &JsValue);
        }

        let scope = use_current_scope();
        let cb = move || {
            if is_alive.get() {
                // Scope is still valid. We can call the callback.
                scope.run_in(f);
            }
        };
        queue_microtask(&Closure::once_into_js(cb));
    }
}

/// Get `window.document`.
pub(crate) fn document() -> web_sys::Document {
    thread_local! {
        /// Cache document since it is frequently accessed to prevent going through js-interop.
        static DOCUMENT: web_sys::Document = web_sys::window().unwrap_throw().document().unwrap_throw();
    };
    DOCUMENT.with(|document| document.clone())
}

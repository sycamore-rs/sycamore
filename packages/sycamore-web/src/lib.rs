//! # `sycamore-web`
//!
//! Web rendering backend for [`sycamore`](https://docs.rs/sycamore). This is already re-exported
//! in the main `sycamore` crate, so you should rarely need to use this crate directly.
//!
//! ## Feature flags
//!
//! - `hydrate` - Enables hydration support in DOM node. By default, hydration is disabled to reduce
//!   binary size.
//!
//! - `suspense` - Enables suspense support.
//!
//! - `wasm-bindgen-interning` (_default_) - Enables interning for `wasm-bindgen` strings. This
//!   improves performance at a slight cost in binary size. If you want to minimize the size of the
//!   resulting `.wasm` binary, you might want to disable this.
//!
//! ## Server Side Rendering
//!
//! This crate uses target detection to determine whether to use DOM or SSR as the rendering
//! backend. If the target arch is `wasm32`, DOM rendering will be used. Otherwise, SSR will be
//! used. Sometimes, this isn't desirable (e.g., if using server side wasm). To override this
//! behavior, you can set `--cfg sycamore_force_ssr` in your `RUSTFLAGS` environment variable when
//! compiling to force SSR mode even on `wasm32`.

// NOTE: Determining whether we are in SSR mode or not uses the cfg_ssr! and cfg_not_ssr! macros.
// For dependencies, we have to put in the conditions manually.

use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use sycamore_macro::*;
use sycamore_reactive::*;
use wasm_bindgen::prelude::*;

pub mod bind;
pub mod events;

mod components;
mod elements;
mod iter;
mod maybe_dyn;
mod node;
mod noderef;
mod portal;
mod stable_counter;
#[cfg(feature = "suspense")]
mod suspense;
mod view;

pub use self::components::*;
pub use self::elements::*;
pub use self::iter::*;
pub use self::maybe_dyn::*;
pub use self::node::*;
pub use self::noderef::*;
pub use self::portal::*;
pub use self::stable_counter::*;
#[cfg(feature = "suspense")]
pub use self::suspense::*;
pub use self::view::*;

/// We add this to make the macros from `sycamore-macro` work properly.
extern crate self as sycamore;

#[doc(hidden)]
pub mod rt {
    pub use sycamore_core::*;
    #[cfg(feature = "suspense")]
    pub use sycamore_futures::*;
    pub use sycamore_macro::*;
    pub use sycamore_reactive::*;
    #[allow(unused_imports)] // Needed for macro support.
    pub use web_sys;

    #[cfg(feature = "suspense")]
    pub use crate::WrapAsync;
    pub use crate::{bind, custom_element, tags, View};
}

/// A macro that expands to whether we are in SSR mode or not.
///
/// Can also be used with a block to only include the code inside the block if in SSR mode.
///
/// # Example
/// ```
/// # use sycamore_web::*;
/// # fn access_database() {}
/// if is_ssr!() {
///     println!("We are running on the server!");
/// }
///
/// is_ssr! {
///     // Access server only APIs in here.
///     let _ = access_database();
/// }
/// ```
#[macro_export]
macro_rules! is_ssr {
    () => {
        cfg!(any(not(target_arch = "wasm32"), sycamore_force_ssr))
    };
    ($($tt:tt)*) => {
        #[cfg(any(not(target_arch = "wasm32"), sycamore_force_ssr))]
        { $($tt)* }
    };
}

/// A macro that expands to whether we are in DOM mode or not.
///
/// Can also be used with a block to only include the code inside the block if in DOM mode.
///
/// # Example
/// ```
/// # use sycamore_web::*;
/// if is_not_ssr!() {
///     console_log!("We are running in the browser!");
/// }
///
/// is_not_ssr! {
///     // Access browser only APIs in here.
///     let document = document();
/// }
/// ```
#[macro_export]
macro_rules! is_not_ssr {
    () => {
        !$crate::is_ssr!()
    };
    ($($tt:tt)*) => {
        #[cfg(all(target_arch = "wasm32", not(sycamore_force_ssr)))]
        { $($tt)* }
    };
}

/// `macro_rules!` equivalent of [`cfg_ssr`]. This is to get around the limitation of not being
/// able to put proc-macros on `mod` items.
#[macro_export]
macro_rules! cfg_ssr_item {
    ($item:item) => {
        #[cfg(any(not(target_arch = "wasm32"), sycamore_force_ssr))]
        $item
    };
}

/// `macro_rules!` equivalent of [`cfg_not_ssr`]. This is to get around the limitation of not being
/// able to put proc-macros on `mod` items.
#[macro_export]
macro_rules! cfg_not_ssr_item {
    ($item:item) => {
        #[cfg(all(target_arch = "wasm32", not(sycamore_force_ssr)))]
        $item
    };
}

/// A type alias for the rendering backend.
#[cfg_ssr]
pub type HtmlNode = SsrNode;
/// A type alias for the rendering backend.
#[cfg_not_ssr]
#[cfg(not(feature = "hydrate"))]
pub type HtmlNode = DomNode;
/// A type alias for the rendering backend.
#[cfg_not_ssr]
#[cfg(feature = "hydrate")]
pub type HtmlNode = HydrateNode;

/// A type alias for [`Children`](sycamore_core::Children) automatically selecting the correct node
/// type.
pub type Children = sycamore_core::Children<View>;

/// A struct for keeping track of state used for hydration.
#[derive(Debug, Clone, Copy)]
struct HydrationRegistry {
    next_key: Signal<u32>,
}

impl HydrationRegistry {
    pub fn new() -> Self {
        HydrationRegistry {
            next_key: create_signal(0),
        }
    }

    /// Get the next hydration key and increment the internal state. This new key will be unique.
    #[allow(unused, reason = "Unused in DOM mode.")]
    pub fn next_key(self) -> u32 {
        let key = self.next_key.get();
        self.next_key.set(key + 1);
        key
    }
}

impl Default for HydrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new effect, but only if we are not in SSR mode.
pub fn create_client_effect(f: impl FnMut() + 'static) {
    if is_not_ssr!() {
        create_effect(f);
    }
}

/// Queue up a callback to be executed when the component is mounted.
///
/// If not on `wasm32` target, does nothing.
///
/// # Potential Pitfalls
///
/// If called inside an async-component, the callback will be called after the next suspension
/// point (when there is an `.await`).
pub fn on_mount(f: impl FnOnce() + 'static) {
    if cfg!(target_arch = "wasm32") {
        let is_alive = Rc::new(Cell::new(true));
        on_cleanup({
            let is_alive = Rc::clone(&is_alive);
            move || is_alive.set(false)
        });

        let scope = use_current_scope();
        let cb = move || {
            if is_alive.get() {
                scope.run_in(f);
            }
        };
        queue_microtask(cb);
    }
}

/// Alias for `queueMicrotask`.
pub fn queue_microtask(f: impl FnOnce() + 'static) {
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = "queueMicrotask")]
        fn queue_microtask_js(f: &wasm_bindgen::JsValue);
    }
    queue_microtask_js(&Closure::once_into_js(f));
}

/// Utility function for accessing the global [`web_sys::Window`] object.
pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

/// Utility function for accessing the global [`web_sys::Document`] object.
pub fn document() -> web_sys::Document {
    thread_local! {
        /// Cache for small performance improvement by preventing repeated calls to `window().document()`.
        static DOCUMENT: web_sys::Document = window().document().expect("no `document` exists");
    }
    DOCUMENT.with(Clone::clone)
}

/// Log a message to the JavaScript console if on wasm32. Otherwise logs it to stdout.
///
/// Note: this does not work properly for server-side WASM since it will mistakenly try to log to
/// the JS console.
#[macro_export]
macro_rules! console_log {
    ($($arg:tt)*) => {
        if is_not_ssr!() {
            $crate::rt::web_sys::console::log_1(&::std::format!($($arg)*).into());
        } else {
            ::std::println!($($arg)*);
        }
    };
}

/// Log a warning to the JavaScript console if on wasm32. Otherwise logs it to stderr.
///
/// Note: this does not work properly for server-side WASM since it will mistakenly try to log to
/// the JS console.
#[macro_export]
macro_rules! console_warn {
    ($($arg:tt)*) => {
        if is_not_ssr!() {
            $crate::rt::web_sys::console::warn_1(&::std::format!($($arg)*).into());
        } else {
            ::std::eprintln!($($arg)*);
        }
    };
}

/// Prints an error message to the JavaScript console if on wasm32. Otherwise logs it to stderr.
///
/// Note: this does not work properly for server-side WASM since it will mistakenly try to log to
/// the JS console.
#[macro_export]
macro_rules! console_error {
    ($($arg:tt)*) => {
        if is_not_ssr!() {
            $crate::rt::web_sys::console::error_1(&::std::format!($($arg)*).into());
        } else {
            ::std::eprintln!($($arg)*);
        }
    };
}

/// Debug the value of a variable to the JavaScript console if on wasm32. Otherwise logs it to
/// stdout.
///
/// Note: this does not work properly for server-side WASM since it will mistakenly try to log to
/// the JS console.
#[macro_export]
macro_rules! console_dbg {
    () => {
        if is_not_ssr!() {
            $crate::rt::web_sys::console::log_1(
                &::std::format!("[{}:{}]", ::std::file!(), ::std::line!(),).into(),
            );
        } else {
            ::std::dbg!($arg);
        }
    };
    ($arg:expr $(,)?) => {
        if is_not_ssr!() {
            $crate::rt::web_sys::console::log_1(
                &::std::format!(
                    "[{}:{}] {} = {:#?}",
                    ::std::file!(),
                    ::std::line!(),
                    ::std::stringify!($arg),
                    $arg
                )
                .into(),
            );
        } else {
            ::std::dbg!($arg);
        }
    };
    ($($arg:expr),+ $(,)?) => {
        $($crate::console_dbg!($arg);)+
    }
}

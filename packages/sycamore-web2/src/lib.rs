#[cfg(feature = "dom")]
mod dom;
mod elements;
pub mod events;
mod iter;
mod node;
mod portal;
#[cfg(feature = "ssr")]
mod ssr;
mod view;

#[cfg(feature = "dom")]
pub use dom::*;
pub use elements::*;
pub use iter::*;
pub use node::*;
pub use portal::*;
#[cfg(feature = "ssr")]
pub use ssr::*;
pub use view::*;

use std::{
    any::{Any, TypeId},
    borrow::Cow,
    cell::{OnceCell, RefCell},
    rc::Rc,
};

pub use sycamore_reactive::*;
use wasm_bindgen::JsCast;

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

/// Marker struct to be inserted into reactive context to indicate that we are in SSR mode.
#[derive(Clone, Copy)]
struct SsrMode;

/// Returns whether we are in SSR mode or not.
pub fn is_ssr() -> bool {
    if cfg!(feature = "dom") && !cfg!(feature = "ssr") {
        false
    } else if cfg!(feature = "ssr") && !cfg!(feature = "dom") {
        true
    } else {
        // Do a runtime check.
        try_use_context::<SsrMode>().is_some()
    }
}

/// Returns whether we are in client side rendering (CSR) mode or not.
///
/// This is the opposite of [`is_ssr`].
pub fn is_client() -> bool {
    !is_ssr()
}

/// Create a new effect, but only if we are not in SSR mode.
pub fn create_client_effect(f: impl FnMut() + 'static) {
    if !is_ssr() {
        create_effect(f);
    }
}

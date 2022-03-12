//! Utilities for Sycamore when working with futures and async.
//!
//! # Spawning futures
//!
//! The recommended way to spawn a future is to use the
//! [`spawn_local`](ScopeSpawnLocal::spawn_local) method on the reactive scope. The benefit of using
//! this instead of [`wasm_bindgen_futures::spawn_local`] is that the future does not need to be
//! `'static`, allowing values in the surrounding scope to be directly referenced from inside the
//! future without any cloning necessary.
//!
//! # Suspense
//!
//! To find out more about suspense, read the [docs for the suspense module](crate::suspense).

use std::future::Future;

// Re-export `wasm_bindgen_futures` crate.
pub use wasm_bindgen_futures::*;
// Re-export `sycamore-futures` crate.
pub use sycamore_futures::*;

use crate::prelude::*;

/// Extension trait for [`Scope`] adding the [`create_resource`](ScopeFuturesExt::create_resource)
/// method.
pub trait ScopeFuturesExt<'a> {
    /// Create a new async resource.
    ///
    /// TODO: docs + example
    #[deprecated = "use Scope::spawn_local instead"]
    fn create_resource<U, F>(self, f: F) -> RcSignal<Option<U>>
    where
        U: 'static,
        F: Future<Output = U> + 'static;
}

impl<'a> ScopeFuturesExt<'a> for Scope<'a> {
    fn create_resource<U, F>(self, f: F) -> RcSignal<Option<U>>
    where
        U: 'static,
        F: Future<Output = U> + 'static,
    {
        let signal = create_rc_signal(None);

        spawn_local({
            let signal = signal.clone();
            async move {
                signal.set(Some(f.await));
            }
        });

        signal
    }
}

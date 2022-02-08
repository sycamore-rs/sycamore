//! Utilities for Sycamore when working with futures and async.

use std::future::Future;

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
    fn create_resource<U, F>(&'a self, f: F) -> RcSignal<Option<U>>
    where
        U: 'static,
        F: Future<Output = U> + 'static;
}

impl<'a> ScopeFuturesExt<'a> for Scope<'a> {
    fn create_resource<U, F>(&'a self, f: F) -> RcSignal<Option<U>>
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

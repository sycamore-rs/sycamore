//! Utilities for Sycamore when working with futures and async.
//!
//! # Spawning futures
//!
//! The recommended way to spawn a future is to use the
//! [`spawn_local_scoped`] method on the reactive scope. The benefit of using
//! this instead of [`wasm_bindgen_futures::spawn_local`] is that the future does not need to be
//! `'static`, allowing values in the surrounding scope to be directly referenced from inside the
//! future without any cloning necessary.
//!
//! # Suspense
//!
//! To find out more about suspense, read the [docs for the suspense module](crate::suspense).

use std::future::Future;

// Re-export `sycamore-futures` crate.
pub use sycamore_futures::*;
// Re-export `wasm_bindgen_futures` crate.
pub use wasm_bindgen_futures::*;

use crate::prelude::*;

/// Create a new async resource.
///
/// TODO: docs + example

pub fn create_resource<'a, U: 'a, F>(cx: Scope<'a>, f: F) -> RcSignal<Option<U>>
where
    F: Future<Output = U> + 'a,
{
    let signal = create_rc_signal(None);

    spawn_local_scoped(cx, {
        let signal = signal.clone();
        async move {
            signal.set(Some(f.await));
        }
    });

    signal
}

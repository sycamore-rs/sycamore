use std::future::Future;

pub use wasm_bindgen_futures::*;
// Re-export `sycamore-futures` crate.
pub use sycamore_futures::*;

use crate::prelude::*;

pub trait ScopeFuturesExt<'a> {
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

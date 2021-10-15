use std::future::Future;

use sycamore_reactive::current_scope;
use wasm_bindgen_futures::spawn_local;

/// A wrapper around [`wasm_bindgen_futures::spawn_local`] that extends the current reactive scope
/// that it is called in.
///
/// If the scope is dropped by the time the future is spawned, the callback will not be called.
///
/// If not on `wasm32` target arch, this function is a no-op.
///
/// # Panics
/// This function panics if called outside of a reactive scope.
///
/// # Example
/// ```
/// use sycamore::futures::spawn_local_in_scope;
/// use sycamore::prelude::*;
///
/// create_root(|| {
///     // Inside reactive scope.
///     spawn_local_in_scope(async {
///         // Still inside reactive scope.
///     });
/// });
/// ```
pub fn spawn_local_in_scope<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    if cfg!(target_arch = "wasm32") {
        if let Some(scope) = current_scope() {
            spawn_local(async move {
                scope.extend_future(future).await;
            });
        } else {
            panic!("spawn_local_in_scope called outside of reactive scope");
        }
    }
}

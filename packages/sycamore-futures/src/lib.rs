//! Futures support for reactive scopes.

#![deny(missing_debug_implementations)]

use futures::future::abortable;
use futures::Future;
use sycamore_reactive3::on_cleanup;

/// If running on `wasm32` target, does nothing. Otherwise creates a new `tokio::task::LocalSet`
/// scope.
///
/// Normally, you do not need to call this as it is handled internally by Sycamore when creating
/// your app.
pub async fn provide_executor_scope<U>(f: impl Future<Output = U>) -> U {
    #[cfg(target_arch = "wasm32")]
    {
        f.await
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let local = tokio::task::LocalSet::new();
        local.run_until(f).await
    }
}

/// Spawns a `!Send` future on the current scope. If the scope is destroyed before the future is
/// completed, it is aborted immediately. This ensures that it is impossible to access any
/// values referencing the scope after they are destroyed.
pub fn spawn_local_scoped(f: impl Future<Output = ()> + 'static) {
    let (abortable, handle) = abortable(f);
    on_cleanup(move || handle.abort());
    #[cfg(not(target_arch = "wasm32"))]
    tokio::task::spawn_local(abortable);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let _ = abortable.await;
    });
}

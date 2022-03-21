//! Futures support for reactive scopes.

use std::pin::Pin;

use futures::future::abortable;
use futures::Future;
use sycamore_reactive::{on_cleanup, Scope};

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

/// Extension trait for Sycamore, providing the [`spawn_local`](ScopeSpawnLocal::spawn_local)
/// method.
pub trait ScopeSpawnLocal<'a> {
    /// Spawns a `!Send` future on the current scope. If the scope is destroyed before the future is
    /// completed, it is aborted immediately. This ensures that it is impossible to access any
    /// values referencing the scope after they are destroyed.
    fn spawn_local(self, f: impl Future<Output = ()> + 'a);
}

impl<'a> ScopeSpawnLocal<'a> for Scope<'a> {
    fn spawn_local(self, f: impl Future<Output = ()> + 'a) {
        let boxed: Pin<Box<dyn Future<Output = ()> + 'a>> = Box::pin(f);
        // SAFETY: We are just transmuting the lifetime here so that we can spawn the future.
        // This is safe because we wrap the future in an `Abortable` future which will be
        // immediately aborted once the reactive scope is dropped.
        let extended: Pin<Box<dyn Future<Output = ()> + 'static>> =
            unsafe { std::mem::transmute(boxed) };
        let (abortable, handle) = abortable(extended);
        on_cleanup(self, move || handle.abort());
        #[cfg(not(target_arch = "wasm32"))]
        tokio::task::spawn_local(abortable);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let _ = abortable.await;
        });
    }
}

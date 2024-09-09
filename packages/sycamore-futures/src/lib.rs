//! Futures support for reactive scopes.

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

mod resource;
mod suspense;

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::abortable;
use futures::stream::Abortable;
use futures::Future;
use pin_project::pin_project;
use sycamore_reactive::{on_cleanup, use_current_scope, NodeHandle};

pub use self::resource::*;
pub use self::suspense::*;

/// If running on `wasm32` target, does nothing. Otherwise creates a new `tokio::task::LocalSet`
/// scope.
///
/// Normally, you do not need to call this as it is handled internally by Sycamore when creating
/// your app.
pub async fn provide_executor_scope<U>(f: impl Future<Output = U>) -> U {
    #[cfg(all(target_arch = "wasm32", not(sycamore_force_ssr)))]
    {
        f.await
    }
    #[cfg(any(not(target_arch = "wasm32"), sycamore_force_ssr))]
    {
        let local = tokio::task::LocalSet::new();
        local.run_until(f).await
    }
}

/// Spawns a `!Send` future on the current scope.
///
/// If the scope is destroyed before the future is completed, it is aborted immediately. This
/// ensures that it is impossible to access any values referencing the scope after they are
/// destroyed.
pub fn spawn_local_scoped(f: impl Future<Output = ()> + 'static) {
    let fut = ScopedFuture::new_in_current_scope(f);
    #[cfg(any(not(target_arch = "wasm32"), sycamore_force_ssr))]
    tokio::task::spawn_local(fut);
    #[cfg(all(target_arch = "wasm32", not(sycamore_force_ssr)))]
    wasm_bindgen_futures::spawn_local(async move {
        let _ = fut.await;
    });
}

/// A wrapper that runs the future on the current scope.
#[pin_project]
struct ScopedFuture<T> {
    #[pin]
    task: Abortable<T>,
    scope: NodeHandle,
}

impl<T: Future> Future for ScopedFuture<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.scope.run_in(move || this.task.poll(cx).map(|_| ()))
    }
}

impl<T: Future> ScopedFuture<T> {
    pub fn new_in_current_scope(f: T) -> Self {
        let (abortable, handle) = abortable(f);
        on_cleanup(move || handle.abort());

        let scope = use_current_scope();

        Self {
            task: abortable,
            scope,
        }
    }
}

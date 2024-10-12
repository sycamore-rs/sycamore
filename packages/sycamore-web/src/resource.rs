//! Async resources integrted with suspense.

use std::future::Future;
use std::ops::Deref;

use futures::future::{FutureExt, LocalBoxFuture};
use sycamore_futures::{SuspenseScope, SuspenseTaskGuard};

use crate::*;

/// Represents a asynchronous resource.
#[derive(Clone, Copy)]
pub struct Resource<T: 'static> {
    /// The current value of the resource.
    ///
    /// This will initially be `None` while the resource is first fetched. For subsequent fetches,
    /// the resource will still contain the previous value while the new value is fetched.
    value: Signal<Option<T>>,
    /// Whether the resource is currently loading or not.
    is_loading: Signal<bool>,
    /// The function that fetches the resource.
    #[allow(clippy::complexity)]
    refetch: Signal<Box<dyn FnMut() -> LocalBoxFuture<'static, T>>>,
    /// A list of all the suspense scopes in which the resource is accessed.
    scopes: Signal<Vec<SuspenseScope>>,
    /// A list of suspense guards that are currently active.
    guards: Signal<Vec<SuspenseTaskGuard>>,
}

impl<T: 'static> Resource<T> {
    /// Create a new resource. By itself, this doesn't do anything.
    fn new<F, Fut>(mut refetch: F) -> Self
    where
        F: FnMut() -> Fut + 'static,
        Fut: Future<Output = T> + 'static,
    {
        Self {
            value: create_signal(None),
            is_loading: create_signal(true),
            refetch: create_signal(Box::new(move || refetch().boxed_local())),
            scopes: create_signal(Vec::new()),
            guards: create_signal(Vec::new()),
        }
    }

    /// Attach handlers to call the refetch function on the client side.
    fn fetch_on_client(self) -> Self {
        if is_not_ssr!() {
            create_effect(move || {
                self.is_loading.set(true);
                // Take all the scopes and create a new guard.
                for scope in self.scopes.take() {
                    let guard = SuspenseTaskGuard::from_scope(scope);
                    self.guards.update(|guards| guards.push(guard));
                }

                let fut = self.refetch.update_silent(|f| f());

                sycamore_futures::create_suspense_task(async move {
                    let value = fut.await;
                    batch(move || {
                        self.value.set(Some(value));
                        self.is_loading.set(false);
                        // Now, drop all the guards to resolve suspense.
                        self.guards.update(|guards| guards.clear());
                    });
                });
            })
        }

        self
    }

    /// Returns whether we are currently loading a new value or not.
    pub fn is_loading(&self) -> bool {
        self.is_loading.get()
    }
}

/// Hijack deref so that we can track where the resource is being accessed.
impl<T: 'static> Deref for Resource<T> {
    type Target = ReadSignal<Option<T>>;

    fn deref(&self) -> &Self::Target {
        // If we are already loading, add a new suspense guard. Otherwise, register the scope so
        // that we can create a new guard when loading.
        if self.is_loading.get() {
            let guard = SuspenseTaskGuard::new();
            self.guards.update(|guards| guards.push(guard));
        } else if let Some(scope) = try_use_context::<SuspenseScope>() {
            self.scopes.update(|scopes| scopes.push(scope));
        }

        &self.value
    }
}

/// Create a resrouce that will only be resolved on the client side.
///
/// If the resource has any dependencies, it is recommended to use [`on`] to make them explicit.
/// This will ensure that the dependencies are tracked since reactive variables inside async
/// contexts are not tracked automatically.
///
/// On the server, the resource will always be marked as loading.
pub fn create_client_resource<F, Fut, T>(f: F) -> Resource<T>
where
    F: FnMut() -> Fut + 'static,
    Fut: Future<Output = T> + 'static,
    T: 'static,
{
    Resource::new(f).fetch_on_client()
}

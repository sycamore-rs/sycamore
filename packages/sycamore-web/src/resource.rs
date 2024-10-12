//! Async resources integrted with suspense.

use std::future::Future;
use std::ops::Deref;

use futures::channel::oneshot;
use futures::future::{FutureExt, LocalBoxFuture};

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
    /// A list of oneshot channels that will be resolved when the resource is done fetching.
    tx_finished: Signal<Vec<oneshot::Sender<()>>>,
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
            tx_finished: create_signal(Vec::new()),
        }
    }

    /// Attach handlers to call the refetch function on the client side.
    fn fetch_on_client(self) -> Self {
        if is_not_ssr!() {
            create_effect(move || {
                self.is_loading.set(true);
                let fut = self.refetch.update_silent(|f| f());

                sycamore_futures::create_suspense_task(async move {
                    let value = fut.await;
                    batch(move || {
                        self.value.set(Some(value));
                        self.is_loading.set(false);
                    });
                    // Resolve suspense boundaries.
                    for tx in self.tx_finished.take_silent() {
                console_dbg!("done");
                        tx.send(()).unwrap();
                    }
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
        // If we are currently loading the resource, create a new suspense task that resolves when
        // the resource is done loaading.
        if self.is_loading.get_untracked() {
            let (tx, rx) = oneshot::channel();
            sycamore_futures::create_suspense_task(async move {
                rx.await.unwrap();
                console_dbg!("done2");
            });
            self.tx_finished.update_silent(|vec| vec.push(tx));
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

pub fn create_isomorphic_resource<F, Fut>(f: F)
where
    F: FnOnce() -> Fut + 'static,
    Fut: Future<Output = ()> + 'static,
{
    todo!("isomorphic resources are not implemented yet.")
}

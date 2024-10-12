//! Async resources integrted with suspense.

use std::future::Future;

use crate::*;

/// Represents a asynchronous resource.
pub struct Resource<T: 'static> {
    /// The current value of the resource.
    ///
    /// This will initially be `None` while the resource is first fetched. For subsequent fetches,
    /// the resource will still contain the previous value while the new value is fetched.
    pub value: ReadSignal<Option<T>>,
    is_loading: ReadSignal<bool>,
}

impl<T: 'static> Resource<T> {
    /// Returns whether we are currently loading a new value or not.
    pub fn is_loading(&self) -> bool {
        self.is_loading.get()
    }
}

/// Create a resrouce that will only be resolved on the client side.
///
/// If the resource has any dependencies, it is recommended to use [`on`] to make them explicit.
/// This will ensure that the dependencies are tracked since reactive variables inside async
/// contexts are not tracked automatically.
///
/// On the server, the resource will always be marked as loading.
pub fn create_client_resource<F, Fut, T>(mut f: F) -> Resource<T>
where
    F: FnMut() -> Fut + 'static,
    Fut: Future<Output = T> + 'static,
    T: 'static,
{
    let value = create_signal(None);
    let is_loading = create_signal(true);

    let resource = Resource {
        value: *value,
        is_loading: *is_loading,
    };

    if is_not_ssr!() {
        create_effect(move || {
            // This also tracks all the dependencies of the resource.
            let fut = f();
            let fut = async move {
                is_loading.set(true);
                value.set(Some(fut.await));
                is_loading.set(false);
            };

            #[cfg(feature = "suspense")]
            sycamore_futures::create_suspense_task(fut);
            #[cfg(not(feature = "suspense"))]
            sycamore_futures::spawn_local_scoped(fut);
        });
    }

    resource
}

pub fn create_isomorphic_resource<F, Fut>(f: F)
where
    F: FnOnce() -> Fut + 'static,
    Fut: Future<Output = ()> + 'static,
{
    todo!("isomorphic resources are not implemented yet.")
}

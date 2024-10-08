//! Suspense with first class `async`/`await` support.
//!
//! The [`Suspense`] component is used to "suspend" execution and wait until async tasks are
//! finished before rendering.

use futures::channel::oneshot;
use futures::Future;
use sycamore_reactive::*;

use crate::*;

/// Represents a new suspense scope. This is created by a call to [`await_suspense`].
#[derive(Clone, Debug)]
struct SuspenseScope {
    tasks_remaining: Signal<u32>,
    parent: Option<Box<SuspenseScope>>,
}

impl SuspenseScope {
    /// Create a new suspense scope, optionally with a parent scope.
    ///
    /// The parent scope should always be located in a reactive scope that is an ancestor of
    /// this scope.
    pub fn new(parent: Option<Box<SuspenseScope>>) -> Self {
        Self {
            tasks_remaining: create_signal(0),
            parent,
        }
    }

    /// Returns whether we are currently loading this suspense or not.
    ///
    /// Implementation for the [`use_is_loading`] hook.
    pub fn is_loading(&self) -> bool {
        self.tasks_remaining.get() > 0
            || self
                .parent
                .as_ref()
                .map_or(false, |parent| parent.is_loading())
    }

    /// Returns a future that resolves once the scope is no longer loading.
    pub async fn await_scope(self) {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);
        create_effect(move || {
            if !self.is_loading() {
                if let Some(tx) = tx.take() {
                    tx.send(()).unwrap();
                }
            }
        });

        rx.await.unwrap()
    }
}

/// Submits a new task that is to be tracked by the suspense system.
///
/// This is used to signal to a `Suspense` component higher up in the component hierarchy that
/// there is some async task that should be awaited before showing the UI.
///
/// If this is called from outside a suspense scope, the task will be executed normally.
pub fn submit_suspense_task(f: impl Future<Output = ()> + 'static) {
    if let Some(mut scope) = try_use_context::<SuspenseScope>() {
        scope.tasks_remaining += 1;
        spawn_local_scoped(async move {
            f.await;
            scope.tasks_remaining -= 1;
        });
    } else {
        spawn_local_scoped(f)
    }
}

/// Calls the given function and returns a tuple with the result and a future that resolves when
/// all suspense tasks created within the function are completed.
///
/// If this is called inside another call to [`await_suspense`], this suspense will wait until the
/// parent suspense is resolved.
pub fn await_suspense<T>(f: impl FnOnce() -> T) -> (T, impl Future<Output = ()>) {
    let parent = try_use_context::<SuspenseScope>();
    let scope = SuspenseScope::new(parent.map(Box::new));
    provide_context_in_new_scope(scope.clone(), move || {
        let ret = f();
        let fut = scope.await_scope();
        (ret, fut)
    })
}

/// Waits until all suspense task in current scope are completed.
///
/// Does not create a new suspense scope.
///
/// If not called inside a suspense scope, the future will resolve immediately.
pub async fn await_suspense_current() {
    if let Some(scope) = try_use_context::<SuspenseScope>() {
        scope.await_scope().await;
    }
}

/// Returns whether we are currently loading this suspense or not.
///
/// This will return true if there are any tasks remaining in this scope or in any parent
/// scope.
///
/// This function is also reactive and so the loading state can be tracked. If it is called outside
/// of a suspense scope, it will always return `false`.
pub fn use_is_loading() -> bool {
    try_use_context::<SuspenseScope>().map_or(false, |scope| scope.is_loading())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn suspense_scope() {
        let _ = create_root(|| {
            let _ = await_suspense(|| {
                let outer_scope = try_use_context::<SuspenseScope>();
                assert!(outer_scope.is_some());
                assert!(outer_scope.unwrap().parent.is_none());

                let _ = await_suspense(|| {
                    let inner_scope = try_use_context::<SuspenseScope>();
                    assert!(inner_scope.is_some());
                    assert!(inner_scope.unwrap().parent.is_some());
                });
            });
        });
    }

    #[tokio::test]
    async fn suspense_await_suspense() {
        let (tx, rx) = oneshot::channel();
        let is_completed = Rc::new(Cell::new(false));

        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let _ = create_root({
                    let is_completed = is_completed.clone();
                    || {
                        spawn_local_scoped(async move {
                            let (_, fut) = await_suspense(|| {
                                submit_suspense_task(async move {
                                    rx.await.unwrap();
                                });
                            });

                            fut.await;
                            is_completed.set(true);
                        });
                    }
                });
            })
            .await;

        assert!(!is_completed.get());

        tx.send(()).unwrap();
        local.await;
        assert!(is_completed.get());
    }
}

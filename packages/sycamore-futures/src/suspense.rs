//! Suspense with first class `async`/`await` support.
//!
//! The [`Suspense`] component is used to "suspend" execution and wait until async tasks are
//! finished before rendering.

use futures::channel::oneshot;
use futures::Future;
use sycamore_reactive::*;

use crate::*;

/// A context value that keeps track of all the signals representing the number of tasks remaining
/// in a suspense scope.
///
/// This is useful for figuring out when all suspense tasks are completed on the page.
#[derive(Clone, Debug, Default)]
struct AllTasksRemaining {
    all_tasks_remaining: Signal<Vec<Signal<u32>>>,
}

/// Represents a new suspense scope. This is created by a call to [`create_suspense_scope`].
#[derive(Clone, Debug)]
pub struct SuspenseScope {
    tasks_remaining: Signal<u32>,
    /// The parent suspense scope of the current scope, if it exists.
    pub parent: Option<Box<SuspenseScope>>,
    /// Signal that is set to `true` when the view is rendered and streamed into the buffer.
    /// This is unused on the client side.
    pub sent: Signal<bool>,
}

impl SuspenseScope {
    /// Create a new suspense scope, optionally with a parent scope.
    ///
    /// The parent scope should always be located in a reactive scope that is an ancestor of
    /// this scope.
    pub fn new(parent: Option<Box<SuspenseScope>>) -> Self {
        let tasks_remaining = create_signal(0);
        let global = use_global_scope().run_in(|| use_context_or_else(AllTasksRemaining::default));
        global
            .all_tasks_remaining
            .update(|vec| vec.push(tasks_remaining));
        Self {
            tasks_remaining,
            parent,
            sent: create_signal(false),
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
    pub async fn until_finished(self) {
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

/// Creates a new task that is to be tracked by the suspense system.
///
/// This is used to signal to a `Suspense` component higher up in the component hierarchy that
/// there is some async task that should be awaited before showing the UI.
///
/// If this is called from outside a suspense scope, the task will be executed normally.
pub fn create_suspense_task(f: impl Future<Output = ()> + 'static) {
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

/// Calls the given function and registers all suspense tasks.
///
/// Returns a tuple containing the return value of the function and the created suspense scope.
///
/// If this is called inside another call to [`await_suspense`], this suspense will wait until the
/// parent suspense is resolved.
pub fn create_suspense_scope<T>(f: impl FnOnce() -> T) -> (T, SuspenseScope) {
    let parent = try_use_context::<SuspenseScope>();
    let scope = SuspenseScope::new(parent.map(Box::new));
    provide_context_in_new_scope(scope.clone(), move || {
        let ret = f();
        (ret, scope)
    })
}

/// Waits until all suspense task in current scope are completed.
///
/// Does not create a new suspense scope.
///
/// If not called inside a suspense scope, the future will resolve immediately.
pub async fn await_suspense_current() {
    if let Some(scope) = try_use_context::<SuspenseScope>() {
        scope.until_finished().await;
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

/// Returns whether any suspense scope is current loading.
///
/// This is unlike [`use_is_loading`] in that it can be called outside of a suspense scope and does
/// not apply to any suspense scope in particular.
pub fn use_is_loading_global() -> bool {
    if let Some(global) = try_use_context::<AllTasksRemaining>() {
        global
            .all_tasks_remaining
            .with(|vec| vec.iter().any(|signal| signal.get() > 0))
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn suspense_scope() {
        let _ = create_root(|| {
            let _ = create_suspense_scope(|| {
                let outer_scope = try_use_context::<SuspenseScope>();
                assert!(outer_scope.is_some());
                assert!(outer_scope.unwrap().parent.is_none());

                let _ = create_suspense_scope(|| {
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
                            let (_, scope) = create_suspense_scope(|| {
                                create_suspense_task(async move {
                                    rx.await.unwrap();
                                });
                            });

                            scope.until_finished().await;
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

    #[tokio::test]
    async fn use_is_loading_global_works() {
        let (tx, rx) = oneshot::channel();

        let local = tokio::task::LocalSet::new();
        let mut root = create_root(|| {});
        local
            .run_until(async {
                root = create_root(|| {
                    let _ = create_suspense_scope(|| {
                        create_suspense_task(async move {
                            rx.await.unwrap();
                        });
                    });
                });
            })
            .await;

        root.run_in(|| {
            assert!(use_is_loading_global());
        });

        tx.send(()).unwrap();
        local.await;

        root.run_in(|| {
            assert!(!use_is_loading_global());
        });
    }
}

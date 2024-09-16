//! Suspense with first class `async`/`await` support.
//!
//! The [`Suspense`] component is used to "suspend" execution and wait until async tasks are
//! finished before rendering.

use futures::channel::oneshot;
use futures::Future;
use sycamore_reactive::*;

use crate::*;

/// Internal context state used by suspense.
#[derive(Clone, Copy, Debug)]
pub struct SuspenseState {
    scopes: Signal<Vec<SuspenseScope>>,
}

#[derive(Clone, Copy, Debug)]
struct SuspenseScope {
    count: Signal<u32>,
    is_done: Signal<bool>,
}

/// Creates a new "suspense scope".
///
/// This scope is used to signal to a `Suspense` component higher up in the component hierarchy
/// that there is some async task that should be awaited before rendering the UI.
///
/// The scope ends when the future is resolved.
pub fn suspense_scope(f: impl Future<Output = ()> + 'static) {
    if let Some(state) = try_use_context::<SuspenseState>() {
        if let Some(mut scope) = state.scopes.get_clone().last().cloned() {
            scope.count += 1;
            spawn_local_scoped(async move {
                f.await;
                scope.count -= 1;
            });
            return;
        }
    }
    spawn_local_scoped(f);
}

/// Calls the given function and returns a tuple with the result and a future that resolves when
/// all suspense tasks created within the function are completed.
///
/// If this is called inside another call to [`await_suspense`], this suspense will wait until the
/// parent suspense is resolved.
pub fn await_suspense<T>(f: impl FnOnce() -> T) -> (T, impl Future<Output = ()>) {
    let state = use_context_or_else(|| SuspenseState {
        scopes: create_signal(Vec::new()),
    });
    // Push a new suspense scope.
    let scope = SuspenseScope {
        count: create_signal(0),
        is_done: create_signal(false),
    };
    state.scopes.update(|scopes| scopes.push(scope));
    let ret = f();
    // We have collected all suspense tasks now. Pop the scope.
    state.scopes.update(|scopes| scopes.pop().unwrap());

    let (tx, rx) = oneshot::channel();
    let mut tx = Some(tx);

    // Check if we have a parent scope. If we do, we need to wait until it is resolved.
    let parent = state.scopes.with(|scopes| scopes.last().copied());

    create_effect(move || {
        if scope.count.get() == 0 && (parent.is_none() || parent.unwrap().is_done.get()) {
            if let Some(tx) = tx.take() {
                tx.send(()).unwrap();
                scope.is_done.set(true);
            }
        }
    });
    (ret, async move {
        rx.await.unwrap();
    })
}

/// Waits until all suspense task in current scope are completed.
///
/// Does not create a new suspense scope.
pub async fn await_suspense_current() {
    let state = use_context_or_else(|| SuspenseState {
        scopes: create_signal(Vec::new()),
    });

    let (tx, rx) = oneshot::channel();
    let mut tx = Some(tx);

    // Check if we have a parent scope. If we do, we need to wait until it is resolved.
    let parent = state.scopes.with(|scopes| scopes.last().copied());

    create_effect(move || {
        if parent.is_none() || parent.unwrap().is_done.get() {
            if let Some(tx) = tx.take() {
                tx.send(()).unwrap();
            }
        }
    });

    rx.await.unwrap();
}

/// Returns whether we have any pending suspense tasks.
pub fn is_pending_suspense() -> bool {
    let Some(state) = try_use_context::<SuspenseState>() else {
        return false;
    };
    let scope = state.scopes.with(|scopes| scopes.last().copied());
    scope.is_none() || scope.unwrap().count.get() > 0
}

/// A struct to handle transitions. Created using [`use_transition`].
#[derive(Clone, Copy, Debug)]
pub struct TransitionHandle {
    is_pending: Signal<bool>,
}

impl TransitionHandle {
    /// Returns whether the transition is currently in progress or not. This value can be tracked
    /// from a listener scope.
    pub fn is_pending(&self) -> bool {
        self.is_pending.get()
    }

    /// Start a transition.
    pub fn start(self, f: impl FnOnce() + 'static, done: impl FnOnce() + 'static) {
        spawn_local_scoped(async move {
            self.is_pending.set(true);
            let (_, suspend) = await_suspense(f);
            suspend.await;
            self.is_pending.set(false);
            done();
        });
    }
}

/// Create a new [TransitionHandle]. This allows executing updates and awaiting until all async
/// tasks are completed.
pub fn use_transition() -> TransitionHandle {
    let is_pending = create_signal(false);

    TransitionHandle { is_pending }
}

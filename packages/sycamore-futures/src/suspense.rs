//! Suspense with first class `async`/`await` support.
//!
//! The [`Suspense`] component is used to "suspend" execution and wait until async tasks are
//! finished before rendering.

use futures::channel::oneshot;
use futures::Future;
use sycamore_reactive::*;

use crate::*;

/// Internal context state used by suspense.
#[derive(Clone, Copy)]
struct SuspenseState {
    async_counts: Signal<Vec<Signal<u32>>>,
}

/// Creates a new "suspense scope".
///
/// This scope is used to signal to a `Suspense` component higher up in the component hierarchy
/// that there is some async task that should be awaited before rendering the UI.
///
/// The scope ends when the future is resolved.
pub fn suspense_scope(f: impl Future<Output = ()> + 'static) {
    if let Some(state) = try_use_context::<SuspenseState>() {
        if let Some(mut count) = state.async_counts.get_clone().last().cloned() {
            count += 1;
            spawn_local_scoped(async move {
                f.await;
                count -= 1;
            });
            return;
        }
    }
    spawn_local_scoped(f);
}

/// Waits until all suspense tasks created within the scope are finished.
/// If called inside an outer suspense scope, this will also make the outer suspense scope suspend
/// until this resolves.
pub async fn await_suspense<U>(f: impl Future<Output = U>) -> U {
    let state = use_context_or_else(|| SuspenseState {
        async_counts: create_signal(Vec::new()),
    });
    // Get the outer suspense state.
    let outer_count = state.async_counts.with(|counts| counts.last().copied());
    // Push a new suspense state.
    let count = create_signal(0);
    state.async_counts.update(|counts| counts.push(count));
    let ready = create_selector(move || count.get() == 0);

    if let Some(mut outer_count) = outer_count {
        outer_count += 1;
    }
    let ret = f.await;
    // Pop the suspense state.
    state.async_counts.update(|counts| counts.pop().unwrap());

    let (sender, receiver) = oneshot::channel();
    let mut sender = Some(sender);

    create_effect(move || {
        if ready.get() {
            if let Some(sender) = sender.take() {
                let _ = sender.send(());
            }
        }
    });
    let _ = receiver.await;
    if let Some(mut outer_count) = outer_count {
        outer_count -= 1;
    }
    ret
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
            await_suspense(async move { f() }).await;
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

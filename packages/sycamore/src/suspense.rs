//! Suspense with first class `async`/`await` support.
//!
//! The [`Suspense`] component is used to "suspend" execution and wait until async tasks are
//! finished before rendering.

use std::cell::{Cell, RefCell};

use futures::channel::oneshot;
use futures::Future;
use sycamore_futures::ScopeSpawnLocal;

use crate::prelude::*;

#[derive(Default)]
struct SuspenseState {
    async_counts: RefCell<Vec<RcSignal<u32>>>,
}

/// Props for [`Suspense`].
#[derive(Prop)]
pub struct SuspenseProps<'a, G: GenericNode> {
    /// The fallback [`View`] to display while the child nodes are being awaited.
    #[builder(default)]
    fallback: View<G>,
    children: Children<'a, G>,
}

/// `Suspense` lets you wait for `async` tasks to complete before rendering the UI. This is useful
/// for asynchronous data-fetching or other asynchronous tasks.
///
/// `Suspense` is deeply integrated with [async components](https://sycamore-rs.netlify.app/docs/basics/components).
/// Async components that are nested under the `Suspense` component will not be rendered until they
/// are resolved. Having multiple async components will have the effect that the final UI will only
/// be rendered once all individual async components are rendered. This is useful for showing a
/// loading indicator while the data is being loaded.
///
/// # Example
/// ```
/// use sycamore::prelude::*;
/// use sycamore::suspense::Suspense;
///
/// #[component]
/// async fn AsyncComp<G: Html>(cx: Scope<'_>) -> View<G> {
///     view! { cx, "Hello Suspense!" }
/// }
///
/// #[component]
/// fn App<G: Html>(cx: Scope) -> View<G> {
///     view! { cx,
///         Suspense {
///             fallback: view! { cx, "Loading..." },
///             AsyncComp {}
///         }
///     }
/// }
/// ```
#[component]
pub fn Suspense<'a, G: GenericNode>(cx: Scope<'a>, props: SuspenseProps<'a, G>) -> View<G> {
    let state = cx.use_context_or_else(SuspenseState::default);
    // Get the outer suspense state.
    let outer_count = state.async_counts.borrow().last().cloned();
    // Push a new suspense state.
    let count = create_rc_signal(0);
    state.async_counts.borrow_mut().push(count.clone());
    let ready = cx.create_selector(move || *count.get() == 0);

    let v = props.children.call(cx);
    // Pop the suspense state.
    state.async_counts.borrow_mut().pop().unwrap();

    if let Some(outer_state) = outer_count {
        outer_state.set(*outer_state.get() + 1);
        // We keep track whether outer_state has already been decremented to prevent it from being
        // decremented twice.
        let completed = cx.create_ref(Cell::new(false));
        cx.create_effect(move || {
            if !completed.get() && *ready.get() {
                outer_state.set(*outer_state.get() - 1);
                completed.set(true);
            }
        });
    }

    view! { cx,
        (if *ready.get() { v.clone() } else { props.fallback.clone() })
    }
}

/// Creates a new "suspense scope". This scope is used to signal to a [`Suspense`] component higher
/// up in the component hierarchy that there is some async task that should be awaited before
/// rendering the UI.
///
/// The scope ends when the future is resolved.
pub fn suspense_scope<'a>(cx: Scope<'a>, f: impl Future<Output = ()> + 'a) {
    if let Some(state) = cx.try_use_context::<SuspenseState>() {
        if let Some(count) = state.async_counts.borrow().last().cloned() {
            count.set(*count.get() + 1);
            cx.spawn_local(async move {
                f.await;
                count.set(*count.get() - 1);
            });
            return;
        }
    }
    cx.spawn_local(f);
}

/// Waits until all suspense tasks created within the scope are finished.
pub async fn await_suspense<U>(cx: Scope<'_>, f: impl Future<Output = U>) -> U {
    let state = cx.use_context_or_else(SuspenseState::default);
    // Get the outer suspense state.
    let outer_count = state.async_counts.borrow().last().cloned();
    // Push a new suspense state.
    let count = create_rc_signal(0);
    state.async_counts.borrow_mut().push(count.clone());
    let ready = cx.create_selector({
        let count = count.clone();
        move || *count.get() == 0
    });

    if let Some(outer_count) = &outer_count {
        outer_count.set(*outer_count.get() + 1);
    }
    let ret = f.await;
    // Pop the suspense state.
    state.async_counts.borrow_mut().pop().unwrap();

    let (sender, receiver) = oneshot::channel();
    let sender = cx.create_ref(RefCell::new(Some(sender)));

    cx.create_effect(move || {
        if *ready.get() {
            if let Some(sender) = sender.take() {
                let _ = sender.send(());
            }
        }
    });
    let _ = receiver.await;
    if let Some(outer_count) = &outer_count {
        outer_count.set(*outer_count.get() - 1);
    }
    ret
}

/// A struct to handle transitions. Created using
/// [`use_transition`](ScopeUseTransition::use_transition).
#[derive(Clone, Copy)]
pub struct TransitionHandle<'a> {
    cx: Scope<'a>,
    is_pending: &'a Signal<bool>,
}

impl<'a> TransitionHandle<'a> {
    /// Returns whether the transition is currently in progress or not. This value can be tracked
    /// from a listener scope.
    pub fn is_pending(&self) -> bool {
        *self.is_pending.get()
    }

    /// Start a transition.
    pub fn start(self, f: impl Fn() + 'a) {
        self.cx.spawn_local(async move {
            self.is_pending.set(true);
            await_suspense(self.cx, async move { f() }).await;
            self.is_pending.set(false);
        });
    }
}

/// Extension trait for [`Scope`] adding the [`use_transition`](ScopeUseTransition::use_transition)
/// method.
pub trait ScopeUseTransition<'a> {
    /// Create a new [TransitionHandle]. This allows executing updates and awaiting until all async
    /// tasks are completed.
    fn use_transition(self) -> &'a TransitionHandle<'a>;
}

impl<'a> ScopeUseTransition<'a> for Scope<'a> {
    fn use_transition(self) -> &'a TransitionHandle<'a> {
        let is_pending = self.create_signal(false);

        self.create_ref(TransitionHandle {
            cx: self,
            is_pending,
        })
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use sycamore_futures::provide_executor_scope;

    use super::*;
    use crate::generic_node::render_to_string_await_suspense;

    #[tokio::test]
    async fn suspense() {
        #[component]
        async fn Comp<G: Html>(cx: Scope<'_>) -> View<G> {
            view! { cx, "Hello Suspense!" }
        }

        let view = provide_executor_scope(async {
            render_to_string_await_suspense(|cx| {
                view! { cx,
                    Suspense {
                        fallback: view! { cx, "Loading..." },
                        Comp {}
                    }
                }
            })
            .await
        })
        .await;
        assert_eq!(view, "Hello Suspense!");
    }

    #[tokio::test]
    async fn transition() {
        provide_executor_scope(async {
            create_scope_immediate(|cx| {
                let trigger = cx.create_signal(());
                let transition = cx.use_transition();
                let _: View<SsrNode> = view! { cx,
                    Suspense {
                        children: Children::new(cx, move |cx| {
                            cx.create_effect(move || {
                                trigger.track();
                                assert!(cx.try_use_context::<SuspenseState>().is_some());
                            });
                            View::empty()
                        })
                    }
                };
                transition.start(|| trigger.set(()));
            });
        })
        .await;
    }
}

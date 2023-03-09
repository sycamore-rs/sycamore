//! Suspense with first class `async`/`await` support.
//!
//! The [`Suspense`] component is used to "suspend" execution and wait until async tasks are
//! finished before rendering.

use std::cell::RefCell;

use futures::channel::oneshot;
use futures::Future;
use sycamore_futures::spawn_local_scoped;

use crate::prelude::*;

/// Internal context state used by suspense.
#[derive(Default)]
struct SuspenseState {
    async_counts: RefCell<Vec<RcSignal<u32>>>,
}

/// Props for [`Suspense`].
#[derive(Props, Debug)]
pub struct SuspenseProps<'a, G: GenericNode> {
    /// The fallback [`View`] to display while the child nodes are being awaited.
    #[prop(default)]
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
///         Suspense(fallback=view! { cx, "Loading..." }) {
///             AsyncComp {}
///         }
///     }
/// }
/// ```
#[component]
pub fn Suspense<'a, G: GenericNode>(cx: Scope<'a>, props: SuspenseProps<'a, G>) -> View<G> {
    let v = create_signal(cx, None);
    // If the Suspense is nested under another Suspense, we want the other Suspense to await this
    // one as well.
    suspense_scope(cx, async move {
        let res = await_suspense(cx, async move { props.children.call(cx) }).await;
        v.set(Some(res));
    });

    view! { cx,
        (if let Some(v) = v.get().as_ref() { v.clone() } else { props.fallback.clone() })
    }
}

/// Creates a new "suspense scope". This scope is used to signal to a [`Suspense`] component higher
/// up in the component hierarchy that there is some async task that should be awaited before
/// rendering the UI.
///
/// The scope ends when the future is resolved.
pub fn suspense_scope<'a>(cx: Scope<'a>, f: impl Future<Output = ()> + 'a) {
    if let Some(state) = try_use_context::<SuspenseState>(cx) {
        if let Some(count) = state.async_counts.borrow().last().cloned() {
            count.set(*count.get() + 1);
            spawn_local_scoped(cx, async move {
                f.await;
                count.set(*count.get() - 1);
            });
            return;
        }
    }
    spawn_local_scoped(cx, f);
}

/// Waits until all suspense tasks created within the scope are finished.
/// If called inside an outer suspense scope, this will also make the outer suspense scope suspend
/// until this resolves.
pub async fn await_suspense<U>(cx: Scope<'_>, f: impl Future<Output = U>) -> U {
    let state = use_context_or_else(cx, SuspenseState::default);
    // Get the outer suspense state.
    let outer_count = state.async_counts.borrow().last().cloned();
    // Push a new suspense state.
    let count = create_rc_signal(0);
    state.async_counts.borrow_mut().push(count.clone());
    let ready = create_selector(cx, {
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
    let mut sender = Some(sender);

    create_effect(cx, move || {
        if *ready.get() {
            if let Some(sender) = sender.take() {
                let _ = sender.send(());
            }
        }
    });
    let _ = receiver.await;
    if let Some(outer_count) = outer_count {
        outer_count.set(*outer_count.get() - 1);
    }
    ret
}

/// A struct to handle transitions. Created using [`use_transition`].
#[derive(Clone, Copy, Debug)]
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
    pub fn start(self, f: impl FnOnce() + 'a, done: impl FnOnce() + 'a) {
        spawn_local_scoped(self.cx, async move {
            self.is_pending.set(true);
            await_suspense(self.cx, async move { f() }).await;
            self.is_pending.set(false);
            done();
        });
    }
}

/// Create a new [TransitionHandle]. This allows executing updates and awaiting until all async
/// tasks are completed.
pub fn use_transition(cx: Scope<'_>) -> &TransitionHandle<'_> {
    let is_pending = create_signal(cx, false);

    // SAFETY: We do not access any referenced data in the Drop implementation for TransitionHandle.
    unsafe { create_ref_unsafe(cx, TransitionHandle { cx, is_pending }) }
}

#[cfg(all(test, feature = "ssr", not(miri)))]
mod tests {
    use sycamore_futures::provide_executor_scope;

    use super::*;
    use crate::web::render_to_string_await_suspense;

    #[tokio::test]
    async fn suspense() {
        #[component]
        async fn Comp<G: Html>(cx: Scope<'_>) -> View<G> {
            view! { cx, "Hello Suspense!" }
        }

        let view = provide_executor_scope(async {
            render_to_string_await_suspense(|cx| {
                view! { cx,
                    Suspense(fallback=view! { cx, "Loading..." }) {
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
            let (sender, receiver) = oneshot::channel();
            let mut sender = Some(sender);
            let disposer = create_scope(|cx| {
                let trigger = create_signal(cx, ());
                let transition = use_transition(cx);
                let _: View<SsrNode> = view! { cx,
                    Suspense(
                        children=Children::new(cx, move |cx| {
                            create_effect(cx, move || {
                                trigger.track();
                                assert!(try_use_context::<SuspenseState>(cx).is_some());
                            });
                            view! { cx, }
                        })
                    )
                };
                let done = create_signal(cx, false);
                transition.start(|| trigger.set(()), || done.set(true));
                create_effect(cx, move || {
                    if *done.get() {
                        sender.take().unwrap().send(()).unwrap();
                    }
                });
            });
            receiver.await.unwrap();
            unsafe { disposer.dispose() };
        })
        .await;
    }
}

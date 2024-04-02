//! Suspense with first class `async`/`await` support.
//!
//! The [`Suspense`] component is used to "suspend" execution and wait until async tasks are
//! finished before rendering.

use std::cell::RefCell;
use std::rc::Rc;

use futures::channel::oneshot;
use futures::Future;
use sycamore_futures::spawn_local_scoped;

use crate::prelude::*;

/// Internal context state used by suspense.
#[derive(Clone, Copy)]
struct SuspenseState {
    async_counts: Signal<Vec<Signal<u32>>>,
}

/// Props for [`Suspense`].
#[derive(Props, Debug)]
pub struct SuspenseProps {
    /// The fallback [`View`] to display while the child nodes are being awaited.
    #[prop(default)]
    fallback: View,
    children: Children,
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
/// async fn AsyncComp<G: Html>() -> View<G> {
///     view! { "Hello Suspense!" }
/// }
///
/// #[component]
/// fn App<G: Html>() -> View<G> {
///     view! {
///         Suspense(fallback=view! { "Loading..." }) {
///             AsyncComp {}
///         }
///     }
/// }
/// ```
#[component]
pub fn Suspense(props: SuspenseProps) -> View {
    let SuspenseProps { fallback, children } = props;
    let mut fallback = Some(fallback);

    let show = create_signal(false);
    let view = Rc::new(RefCell::new(None));
    // If the Suspense is nested under another Suspense, we want the other Suspense to await this
    // one as well.
    suspense_scope({
        let view = Rc::clone(&view);
        async move {
            let res = await_suspense(async move { children.call() }).await;

            *view.borrow_mut() = Some(res);
            show.set(true);
        }
    });

    view! {
        (if show.get() { view.take().unwrap() } else { fallback.take().unwrap() })
    }
}

/// Creates a new "suspense scope". This scope is used to signal to a [`Suspense`] component higher
/// up in the component hierarchy that there is some async task that should be awaited before
/// rendering the UI.
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

#[cfg(all(test, feature = "ssr", not(miri)))]
mod tests {
    use sycamore_futures::provide_executor_scope;

    use super::*;
    use crate::web::render_to_string_await_suspense;

    #[tokio::test]
    async fn suspense() {
        #[component]
        async fn Comp() -> View {
            view! { "Hello Suspense!" }
        }

        let view = provide_executor_scope(async {
            render_to_string_await_suspense(|| {
                view! {
                    Suspense(fallback=view! { "Loading..." }) {
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
            let disposer = create_root(|| {
                let trigger = create_signal(());
                let transition = use_transition();
                let _: View = view! {
                    Suspense(
                        children=Children::new(move || {
                            create_effect(move || {
                                trigger.track();
                                assert!(try_use_context::<SuspenseState>().is_some());
                            });
                            view! { }
                        })
                    )
                };
                let done = create_signal(false);
                transition.start(move || trigger.set(()), move || done.set(true));
                create_effect(move || {
                    if done.get() {
                        sender.take().unwrap().send(()).unwrap();
                    }
                });
            });
            receiver.await.unwrap();
            disposer.dispose();
        })
        .await;
    }
}

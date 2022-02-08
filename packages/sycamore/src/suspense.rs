//! Suspense with first class `async`/`await` support.

use std::cell::{Cell, RefCell};

use futures::channel::oneshot;
use futures::Future;

use crate::context::ContextProvider;
use crate::prelude::*;

#[derive(Clone)]
struct SuspenseState {
    async_count: RcSignal<u32>,
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
/// async fn AsyncComp<G: Html>(ctx: ScopeRef<'_>) -> View<G> {
///     view! { ctx, "Hello Suspense!" }
/// }
///
/// #[component]
/// fn App<G: Html>(ctx: ScopeRef) -> View<G> {
///     view! { ctx,
///         Suspense {
///             fallback: view! { ctx, "Loading..." },
///             AsyncComp {}
///         }
///     }
/// }
/// ```
#[component]
pub fn Suspense<'a, G: GenericNode>(ctx: ScopeRef<'a>, props: SuspenseProps<'a, G>) -> View<G> {
    let outer_state = ctx.try_use_context::<SuspenseState>();

    let state = SuspenseState {
        async_count: create_rc_signal(0),
    };
    let ready = ctx.create_selector({
        let state = state.clone();
        move || *state.async_count.get() == 0
    });
    view! { ctx,
        ContextProvider {
            value: state,
            children: Children::new(move |_| {
                let v = props.children.call(ctx);

                if let Some(outer_state) = outer_state {
                    outer_state
                        .async_count
                        .set(*outer_state.async_count.get() + 1);
                    let completed = ctx.create_ref(Cell::new(false));
                    ctx.create_effect(|| {
                        if !completed.get() && *ready.get() {
                            outer_state
                                .async_count
                                .set(*outer_state.async_count.get() - 1);
                            completed.set(true);
                        }
                    });
                }

                view! { ctx,
                    (if *ready.get() { v.clone() } else { props.fallback.clone() })
                }
            })
        }
    }
}

/// Creates a new "suspense scope". This scope is used to signal to a [`Suspense`] component higher
/// up in the component hierarchy that there is some async task that should be awaited before
/// rendering the UI.
///
/// The scope ends when the returned future is resolved.
pub async fn suspense_scope<U>(ctx: ScopeRef<'_>, f: impl Future<Output = U>) -> U {
    if let Some(state) = ctx.try_use_context::<SuspenseState>() {
        state.async_count.set(*state.async_count.get() + 1);
        let ret = f.await;
        state.async_count.set(*state.async_count.get() - 1);
        ret
    } else {
        f.await
    }
}

/// Waits until all suspense tasks created within the scope are finished.
pub async fn await_suspense<G: GenericNode>(
    ctx: ScopeRef<'_>,
    f: impl Future<Output = View<G>>,
) -> View<G> {
    let state = SuspenseState {
        async_count: create_rc_signal(0),
    };
    // TODO: create a child scope to prevent context clashes
    ctx.provide_context(state.clone());
    let ret = f.await;

    let (sender, receiver) = oneshot::channel();
    let sender = ctx.create_ref(RefCell::new(Some(sender)));

    ctx.create_effect(move || {
        if *state.async_count.get() == 0 {
            if let Some(sender) = sender.take() {
                let _ = sender.send(());
            }
        }
    });
    let _ = receiver.await;
    ret
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use sycamore_futures::provide_executor_scope;

    use crate::generic_node::render_to_string_await_suspense;
    use crate::prelude::*;
    use crate::suspense::Suspense;

    #[tokio::test]
    async fn suspense() {
        #[component]
        async fn Comp<G: Html>(ctx: ScopeRef<'_>) -> View<G> {
            view! { ctx, "Hello Suspense!" }
        }

        let view = provide_executor_scope(async {
            render_to_string_await_suspense(|ctx| {
                view! { ctx,
                    Suspense {
                        fallback: view! { ctx, "Loading..." },
                        Comp {}
                    }
                }
            })
            .await
        })
        .await;
        assert_eq!(view, "Hello Suspense!");
    }
}

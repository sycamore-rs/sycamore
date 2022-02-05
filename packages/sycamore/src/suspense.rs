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
    #[builder(default)]
    fallback: View<G>,
    children: Children<'a, G>,
}

/// TODO: docs
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
    // FIXME: use ContextProvider
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

pub async fn await_suspense<G: GenericNode>(
    ctx: ScopeRef<'_>,
    f: impl Future<Output = View<G>>,
) -> View<G> {
    let state = SuspenseState {
        async_count: create_rc_signal(0),
    };
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

//! Suspense with first class `async`/`await` support.

use futures::Future;

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
    let state = SuspenseState {
        async_count: create_rc_signal(0),
    };
    let ready = ctx.create_selector({
        let state = state.clone();
        move || *state.async_count.get() == 0
    });
    ctx.provide_context(state);
    // FIXME: use ContextProvider
    // view! { ctx,
    //     ContextProvider {
    //         value: state,
    //         children: Children::new(move |_| {
    let v = props.children.call(ctx);

    view! { ctx,
        (if *ready.get() { v.clone() } else { props.fallback.clone() })
    }
    //         })
    //     }
    // }
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

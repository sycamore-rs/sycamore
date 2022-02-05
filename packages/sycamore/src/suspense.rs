//! Suspense with first class `async`/`await` support.

use crate::prelude::*;

struct SuspenseState {}

/// Props for [`Suspense`].
#[derive(Prop)]
pub struct SuspenseProps<'a, G: GenericNode> {
    children: Children<'a, G>,
}

/// TODO: docs
#[component]
pub fn Suspense<'a, G: GenericNode>(ctx: ScopeRef<'a>, props: SuspenseProps<'a, G>) -> View<G> {
    // Provide suspense state.
    let state = SuspenseState {};
    ctx.provide_context(state);

    let children = props.children.call(ctx);
    view! { ctx,
        (children)
    }
}

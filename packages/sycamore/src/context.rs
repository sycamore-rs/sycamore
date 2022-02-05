//! Utility components for providing contexts.

use crate::prelude::*;

#[derive(Prop)]
pub struct ContextProviderProps<'a, T, G: GenericNode> {
    value: T,
    children: Children<'a, G>,
}

#[component]
pub fn ContextProvider<'a, T: 'static, G: GenericNode>(
    ctx: ScopeRef<'a>,
    props: ContextProviderProps<'a, T, G>,
) -> View<G> {
    let mut view = None;
    ctx.create_child_scope(|ctx| {
        ctx.provide_context(props.value);
        // SAFETY: `props.children` takes the same parameter as argument passed to ctx.create_child_scope
        view = Some(
            props
                .children
                .call_with_bounded_scope(unsafe { std::mem::transmute(ctx) }),
        );
    });
    view.unwrap()
}

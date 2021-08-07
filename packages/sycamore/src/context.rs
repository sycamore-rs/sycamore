use sycamore_reactive::create_context_scope;

use crate::prelude::*;

/// Props for [`ContextProvider`].
pub struct ContextProviderProps<T, F, G>
where
    T: 'static,
    F: FnOnce() -> Template<G>,
    G: GenericNode,
{
    pub value: T,
    pub children: F,
}

/// Creates a new [`ReactiveScope`] with a context.
#[component(ContextProvider<G>)]
pub fn context_provider<T, F>(props: ContextProviderProps<T, F, G>) -> Template<G>
where
    T: 'static,
    F: FnOnce() -> Template<G>,
{
    let ContextProviderProps { value, children } = props;

    create_context_scope(value, children)
}

pub use sycamore_reactive::use_context;

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
pub fn context_provider<T, F>(_props: ContextProviderProps<T, F, G>) -> Template<G>
where
    T: 'static,
    F: FnOnce() -> Template<G>,
{
    // let ContextProviderProps { value, children } = props;

    // SCOPES.with(|scopes| {
    //     // Create a new ReactiveScope with a context.
    //     let mut scope = ReactiveScope::default();
    //     scope.context = Some(Box::new(Context { value }));
    //     scopes.borrow_mut().push(scope);
    //     let template = children();
    //     let scope = scopes.borrow_mut().pop().unwrap();
    //     on_cleanup(move || drop(scope));
    //     template
    // })
    todo!();
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use sycamore_reactive::*;

    use super::*;

    #[test]
    #[ignore]
    fn basic_context() {
        sycamore::render_to_string(|| {
            template! {
                ContextProvider(ContextProviderProps {
                    value: 1i32,
                    children: || {
                        let ctx = use_context::<i32>();
                        assert_eq!(ctx, 1);
                        template! {}
                    }
                })
            }
        });
    }
}

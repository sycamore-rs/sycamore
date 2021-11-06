//! Context API.

use sycamore_reactive::create_context_scope;
pub use sycamore_reactive::use_context;

use crate::prelude::*;

/// Props for [`ContextProvider`].
pub struct ContextProviderProps<T, F, G>
where
    T: 'static,
    F: FnOnce() -> View<G>,
    G: GenericNode,
{
    pub value: T,
    pub children: F,
}

/// Creates a new [`ReactiveScope`](crate::reactive::ReactiveScope) with a context.
///
/// If a context of the given type exists already, the existing context will be _shadowed_ within
/// the scope. This means that accessing the context inside the scope using [`use_context`] will
/// return the new value, not the shadowed value. Using [`use_context`] outside of this new context
/// scope will continue to return the old value.
///
/// # Example
/// ```
/// use sycamore::prelude::*;
/// use sycamore::context::{ContextProvider, ContextProviderProps, use_context};
///
/// #[derive(Clone)]
/// struct Counter(Signal<i32>);
///
/// #[component(CounterView<G>)]
/// fn counter_view() -> View<G> {
///     let counter = use_context::<Counter>();
///
///     view! {
///         (counter.0.get())
///     }
/// }
///
/// # #[component(App<G>)]
/// # fn app() -> View<G> {
/// view! {
///     ContextProvider(ContextProviderProps {
///         value: Counter(Signal::new(0)),
///         children: || view! {
///             CounterView()
///         }
///     })
/// }
/// # }
/// ```
#[component(ContextProvider<G>)]
pub fn context_provider<T, F>(props: ContextProviderProps<T, F, G>) -> View<G>
where
    T: 'static,
    F: FnOnce() -> View<G>,
{
    let ContextProviderProps { value, children } = props;

    create_context_scope(value, children)
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use sycamore_reactive::use_context;

    #[test]
    fn basic_context() {
        sycamore::render_to_string(|| {
            view! {
                ContextProvider(ContextProviderProps {
                    value: 1i32,
                    children: || {
                        let ctx = use_context::<i32>();
                        assert_eq!(ctx, 1);
                        view! {}
                    },
                })
            }
        });
    }

    #[test]
    fn context_inside_effect_when_reexecuting() {
        #[component(ContextConsumer<G>)]
        fn context_consumer() -> View<G> {
            let _ctx = use_context::<i32>();
            view! {}
        }

        let trigger = Signal::new(());

        let node = view! {
            ContextProvider(ContextProviderProps {
                value: 1i32,
                children: cloned!((trigger) => move || {
                    view! {
                        ({
                            trigger.get(); // subscribe to trigger
                            view! { ContextConsumer() }
                        })
                    }
                }),
            })
        };
        trigger.set(());
        trigger.set(());

        sycamore::render_to_string(|| node);
    }

    #[test]
    #[should_panic = "context not found for type"]
    fn should_panic_with_unknown_context_type() {
        let _ = use_context::<u32>();
    }

    #[test]
    #[should_panic = "context not found for type"]
    fn should_panic_with_unknown_context_type_inside_scope() {
        let _ = create_root(move || {
            let _ = use_context::<u32>();
        });
    }
}

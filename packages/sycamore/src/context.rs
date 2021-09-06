//! Context API.

use sycamore_reactive::create_context_scope;
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

/// Creates a new [`ReactiveScope`](crate::reactive::ReactiveScope) with a context.
///
/// # Example
/// ```rust
/// use sycamore::prelude::*;
/// use sycamore::context::{ContextProvider, ContextProviderProps, use_context};
///
/// #[derive(Clone)]
/// struct Counter(Signal<i32>);
///
/// #[component(CounterView<G>)]
/// fn counter_view() -> Template<G> {
///     let counter = use_context::<Counter>();
///
///     template! {
///         (counter.0.get())
///     }
/// }
///
/// # #[component(App<G>)]
/// # fn app() -> Template<G> {
/// template! {
///     ContextProvider(ContextProviderProps {
///         value: Counter(Signal::new(0)),
///         children: || template! {
///             CounterView()
///         }
///     })
/// }
/// # }
/// ```
#[component(ContextProvider<G>)]
pub fn context_provider<T, F>(props: ContextProviderProps<T, F, G>) -> Template<G>
where
    T: 'static,
    F: FnOnce() -> Template<G>,
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
            template! {
                ContextProvider(ContextProviderProps {
                    value: 1i32,
                    children: || {
                        let ctx = use_context::<i32>();
                        assert_eq!(ctx, 1);
                        template! {}
                    },
                })
            }
        });
    }

    #[test]
    fn context_inside_effect_when_reexecuting() {
        #[component(ContextConsumer<G>)]
        fn context_consumer() -> Template<G> {
            let _ctx = use_context::<i32>();
            template! {}
        }

        let trigger = Signal::new(());

        let node = template! {
            ContextProvider(ContextProviderProps {
                value: 1i32,
                children: cloned!((trigger) => move || {
                    template! {
                        ({
                            trigger.get(); // subscribe to trigger
                            template! { ContextConsumer() }
                        })
                    }
                }),
            })
        };
        trigger.set(());
        trigger.set(());

        sycamore::render_to_string(|| node);
    }
}

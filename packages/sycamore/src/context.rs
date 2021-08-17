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

/// Creates a new [`ReactiveScope`] with a context.
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
                    }
                })
            }
        });
    }
}

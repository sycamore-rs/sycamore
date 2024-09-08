//! Components for suspense.

use std::cell::RefCell;

use sycamore_futures::{await_suspense, suspense_scope};
use sycamore_macro::{component, view, Props};

use crate::*;

/// Props for [`Suspense`].
#[derive(Props, Debug)]
pub struct SuspenseProps {
    /// The fallback [`View`] to display while the child nodes are being awaited.
    #[prop(default)]
    fallback: View,
    children: Children,
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
/// async fn AsyncComp<G: Html>() -> View<G> {
///     view! { "Hello Suspense!" }
/// }
///
/// #[component]
/// fn App<G: Html>() -> View<G> {
///     view! {
///         Suspense(fallback=view! { "Loading..." }) {
///             AsyncComp {}
///         }
///     }
/// }
/// ```
#[component]
pub fn Suspense(props: SuspenseProps) -> View {
    let SuspenseProps { fallback, children } = props;
    let mut fallback = Some(fallback);

    let show = create_signal(false);
    let view = Rc::new(RefCell::new(None));
    // If the Suspense is nested under another Suspense, we want the other Suspense to await this
    // one as well.
    suspense_scope({
        let view = Rc::clone(&view);
        async move {
            let res = await_suspense(async move { children.call() }).await;

            *view.borrow_mut() = Some(res);
            show.set(true);
        }
    });

    view! {
        (if show.get() { view.take().unwrap() } else { fallback.take().unwrap() })
    }
}

#[cfg(test)]
mod tests {
    use futures::channel::oneshot;
    use sycamore_futures::{provide_executor_scope, use_transition};

    use super::*;

    #[tokio::test]
    async fn suspense() {
        #[component]
        async fn Comp() -> View {
            view! { "Hello Suspense!" }
        }

        let view = provide_executor_scope(async {
            render_to_string_await_suspense(|| {
                view! {
                    Suspense(fallback=view! { "Loading..." }) {
                        Comp {}
                    }
                }
            })
            .await
        })
        .await;
        assert_eq!(view, "Hello Suspense!");
    }

    #[tokio::test]
    async fn transition() {
        provide_executor_scope(async {
            let (sender, receiver) = oneshot::channel();
            let mut sender = Some(sender);
            let disposer = create_root(|| {
                let trigger = create_signal(());
                let transition = use_transition();
                let _: View = view! {
                    Suspense(
                        children=Children::new(move || {
                            create_effect(move || {
                                trigger.track();
                                assert!(try_use_context::<SuspenseState>().is_some());
                            });
                            view! { }
                        })
                    )
                };
                let done = create_signal(false);
                transition.start(move || trigger.set(()), move || done.set(true));
                create_effect(move || {
                    if done.get() {
                        sender.take().unwrap().send(()).unwrap();
                    }
                });
            });
            receiver.await.unwrap();
            disposer.dispose();
        })
        .await;
    }
}

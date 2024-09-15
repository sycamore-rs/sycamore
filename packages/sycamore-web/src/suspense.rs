//! Components for suspense.

use std::future::Future;

use futures::SinkExt;
use sycamore_futures::{await_suspense, spawn_local_scoped, suspense_scope};
use sycamore_macro::{component, Props};

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
/// use sycamore::web::Suspense;
///
/// #[component]
/// async fn AsyncComp() -> View {
///     view! { "Hello Suspense!" }
/// }
///
/// #[component]
/// fn App() -> View {
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

    is_ssr! {
        let mode = use_context::<SsrMode>();
        match mode {
            // In sync mode, we don't even bother about the children and just return the fallback.
            //
            // We make sure to return a closure so that the view can be properly hydrated.
            SsrMode::Sync => View::from(move || fallback.take().unwrap()),
            SsrMode::Blocking => {
                // We need to create a hydration key so that we know which suspense boundary it is
                // when we replace the marker with the suspended content.
                let reg = use_context::<HydrationRegistry>();
                let key = reg.next_key();

                // Push `children` to the suspense fragments lists.
                let (view, suspend) = await_suspense(move || children.call());
                let fragment = SuspenseFragment {
                    key,
                    view,
                    suspend: Box::new(suspend),
                };
                let mut state = use_context::<SuspenseState>();
                spawn_local_scoped(async move {
                    state
                        .sender
                        .send(fragment)
                        .await
                        .expect("could not send suspense fragment");
                });
                View::from(move || SsrNode::SuspenseMarker { key })
            }
            // In streaming mode, we render the fallback and then stream the result of the children
            // once suspense is resolved.
            SsrMode::Streaming => {
                // We need to create a hydration key so that we know which suspense boundary it is
                // when we stream the content.
                //
                // FIXME: does this introduce non-determinism depending on suspense completion
                // order?
                let reg = use_context::<HydrationRegistry>();
                let key = reg.next_key();

                // Push `children` to the suspense fragments lists.
                let (view, suspend) = await_suspense(move || children.call());
                let fragment = SuspenseFragment {
                    key,
                    view,
                    suspend: Box::new(suspend),
                };
                let mut state = use_context::<SuspenseState>();
                spawn_local_scoped(async move {
                    state
                        .sender
                        .send(fragment)
                        .await
                        .expect("could not send suspense fragment");
                });

                View::from(move || fallback.take().unwrap())
            }
        }
    }
    is_not_ssr! {
        let show = create_signal(false);
        let (view, suspend) = await_suspense(move || children.call());
        // If the Suspense is nested under another Suspense, we want the other Suspense to await
        // this one as well.
        suspense_scope(async move {
            suspend.await;
            show.set(true);
        });

        let mut view = Some(utils::wrap_in_document_fragment(view));
        view! {
            (if !show.get() { fallback.take().unwrap() } else { view.take().unwrap() })
        }
    }
}

/// Convert an async component to a regular sync component. Also wraps the async component inside a
/// suspense scope so that content is properly suspended.
#[component]
pub fn WrapAsync<F: Future<Output = View>>(f: impl FnOnce() -> F + 'static) -> View {
    is_not_ssr! {
        let view = create_signal(View::default());
        let ret = view! { ({
            view.track();
            view.update_silent(std::mem::take)
        }) };
        suspense_scope(async move {
            view.set(f().await);
        });
        ret
    }
    is_ssr! {
        use std::cell::RefCell;
        let node = Rc::new(RefCell::new(View::default()));
        suspense_scope({
            let node = Rc::clone(&node);
            async move {
                *node.borrow_mut() = f().await;
            }
        });
        View::from(move || SsrNode::Dynamic {
            view: Rc::clone(&node),
        })
    }
}

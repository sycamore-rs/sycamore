//! Components for suspense.

use std::future::Future;
use std::num::NonZeroU32;

use sycamore_futures::{await_suspense, submit_suspense_task};
use sycamore_macro::{component, Props};

use crate::*;

/// Props for [`Suspense`].
#[derive(Props)]
pub struct SuspenseProps {
    /// The fallback [`View`] to display while the child nodes are being awaited.
    #[prop(default, setter(transform = |f: impl FnOnce() -> View + 'static| Some(Box::new(f) as Box<dyn FnOnce() -> View>)))]
    fallback: Option<Box<dyn FnOnce() -> View>>,
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
///         Suspense(fallback=|| view! { "Loading..." }) {
///             AsyncComp {}
///         }
///     }
/// }
/// ```
#[component]
pub fn Suspense(props: SuspenseProps) -> View {
    let SuspenseProps { fallback, children } = props;
    let fallback = fallback.unwrap_or_else(|| Box::new(View::default));
    let mut fallback = Some(fallback);

    is_ssr! {
        use futures::SinkExt;
        use sycamore_futures::spawn_local;

        let mode = use_context::<SsrMode>();
        match mode {
            // In sync mode, we don't even bother about the children and just return the fallback.
            //
            // We make sure to return a closure so that the view can be properly hydrated.
            SsrMode::Sync => View::from(move || fallback.take().unwrap()()),
            // In blocking mode, we render a marker node and then replace the marker node with the
            // children once the suspense is resolved.
            //
            // In streaming mode, we render the fallback and then stream the result of the children
            // once suspense is resolved.
            SsrMode::Blocking | SsrMode::Streaming => {
                // We need to create a suspense key so that we know which suspense boundary it is
                // when we replace the marker with the suspended content.
                let key = use_suspense_key();

                // Push `children` to the suspense fragments lists.
                let (view, suspend) = await_suspense(move || HydrationRegistry::in_suspense_scope(key, move || children.call()));
                let fragment = SuspenseFragment::new(key, view);
                let mut state = use_context::<SuspenseState>();
                spawn_local(async move {
                    suspend.await;
                    let _ = state.sender.send(fragment).await;
                });

                // Add some marker nodes so that we know start and finish of fallback.
                let start = view! { suspense-start(data-key=key.to_string()) };
                let marker = View::from(move || SsrNode::SuspenseMarker { key: key.into() });
                let end = view! { NoHydrate { suspense-end(data-key=key.to_string()) } };

                let mut fallback = if mode == SsrMode::Blocking {
                    View::from((start, marker, end))
                } else if mode == SsrMode::Streaming {
                    View::from((
                        start,
                        marker,
                        view! { NoHydrate(children=fallback.take().unwrap().into()) },
                        end,
                    ))
                } else {
                    unreachable!()
                };
                View::from(move || std::mem::take(&mut fallback))
            }
        }
    }
    is_not_ssr! {
        let mode = if IS_HYDRATING.get() {
            use_context::<SsrMode>()
        } else {
            SsrMode::Sync
        };
        match mode {
            SsrMode::Sync => {
                let show = create_signal(false);
                let (view, suspend) = await_suspense(move || children.call());
                // If the Suspense is nested under another Suspense, we want the other Suspense to await
                // this one as well.
                submit_suspense_task(async move {
                    suspend.await;
                    show.set(true);
                });

                let mut view = utils::wrap_in_document_fragment(view);
                view! {
                    (if !show.get() { fallback.take().unwrap()() } else { std::mem::take(&mut view) })
                }
            }
            SsrMode::Blocking | SsrMode::Streaming => {
                // Blocking: Since the fallback is never rendered on the server side, we don't need
                // to hydrate it either.
                //
                // Streaming: By the time the WASM is running, page loading should already be completed since
                // WASM runs inside a deferred script. Therefore we only need to hydrate the view
                // and not the fallback.

                // First hydrate the `<sycamore-start>` element to get the suspense scope.
                let start = view! { suspense-start() };
                let node = start.nodes[0].as_web_sys().unchecked_ref::<web_sys::Element>();
                let key: NonZeroU32 = node.get_attribute("data-key").unwrap().parse().unwrap();

                HydrationRegistry::in_suspense_scope(key, move || children.call())
            }
        }
    }
}

/// Convert an async component to a regular sync component. Also wraps the async component inside a
/// suspense scope so that content is properly suspended.
#[component]
pub fn WrapAsync<F: Future<Output = View>>(f: impl FnOnce() -> F + 'static) -> View {
    is_not_ssr! {
        let mode = if IS_HYDRATING.get() {
            use_context::<SsrMode>()
        } else {
            SsrMode::Sync
        };
        match mode {
            SsrMode::Sync => {
                let view = create_signal(View::default());
                let ret = view! { ({
                    view.track();
                    view.update_silent(std::mem::take)
                }) };
                submit_suspense_task(async move {
                    view.set(f().await);
                });
                ret
            }
            SsrMode::Blocking | SsrMode::Streaming => {
                // TODO: This does not properly hydrate dynamic text nodes.
                submit_suspense_task(async move { f().await; });
                view! {}
            }
        }
    }
    is_ssr! {
        use std::sync::{Arc, Mutex};

        let node = Arc::new(Mutex::new(View::default()));
        submit_suspense_task({
            let node = Arc::clone(&node);
            async move {
                *node.lock().unwrap() = f().await;
            }
        });
        View::from(move || SsrNode::Dynamic {
            view: Arc::clone(&node),
        })
    }
}

#[cfg_ssr]
pub(crate) struct SuspenseFragment {
    pub key: NonZeroU32,
    pub view: View,
}

#[cfg_ssr]
impl SuspenseFragment {
    pub fn new(key: NonZeroU32, view: View) -> Self {
        Self { key, view }
    }
}

/// Context for passing suspense fragments in SSR mode.
#[cfg_ssr]
#[derive(Clone)]
pub(crate) struct SuspenseState {
    pub sender: futures::channel::mpsc::Sender<SuspenseFragment>,
}

/// Global counter for providing suspense key.
#[derive(Debug, Clone, Copy)]
struct SuspenseCounter {
    next: Signal<NonZeroU32>,
}

impl SuspenseCounter {
    fn new() -> Self {
        Self {
            next: create_signal(NonZeroU32::new(1).unwrap()),
        }
    }
}

/// Get the next suspense key.
pub fn use_suspense_key() -> NonZeroU32 {
    let global_scope = use_global_scope();
    let counter = global_scope.run_in(|| use_context_or_else(SuspenseCounter::new));

    let next = counter.next.get();
    counter.next.set(next.checked_add(1).unwrap());
    next
}

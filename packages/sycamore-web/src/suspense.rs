//! Components for suspense.

use std::future::Future;
use std::num::NonZeroU32;

use sycamore_futures::{
    create_detatched_suspense_scope, create_suspense_scope, create_suspense_task,
};
use sycamore_macro::{component, Props};

use crate::*;

/// Props for [`Suspense`] and [`Transition`].
#[derive(Props)]
pub struct SuspenseProps {
    /// The fallback [`View`] to display while the child nodes are being awaited.
    #[prop(default = Box::new(|| view! {}), setter(transform = |f: impl Fn() -> View + 'static| Box::new(f) as Box<dyn Fn() -> View>))]
    fallback: Box<dyn Fn() -> View>,
    children: Children,
    /// The component will automatically update this signal with the `is_loading` state.
    ///
    /// This is only updated in non-SSR mode.
    #[prop(default = Box::new(|_| {}), setter(transform = |f: impl FnMut(bool) + 'static| Box::new(f) as Box<dyn FnMut(bool)>))]
    set_is_loading: Box<dyn FnMut(bool) + 'static>,
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
    let SuspenseProps {
        fallback,
        children,
        mut set_is_loading,
    } = props;

    is_ssr! {
        use futures::SinkExt;

        let _ = &mut set_is_loading;

        let mode = use_context::<SsrMode>();
        match mode {
            // In sync mode, we don't even bother about the children and just return the fallback.
            //
            // We make sure to return a closure so that the view can be properly hydrated.
            SsrMode::Sync => view! {
                Show(when=true) {
                    (fallback())
                }
                Show(when=false) {}
            },
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
                let (mut view, suspense_scope) = create_suspense_scope(move || HydrationRegistry::in_suspense_scope(key, move || children.call()));
                let state = use_context::<SuspenseState>();
                // TODO: error if scope is destroyed before suspense resolves.
                // Probably can fix this by using `FuturesOrdered` instead.
                sycamore_futures::spawn_local_scoped(async move {
                    suspense_scope.until_finished().await;
                    debug_assert!(!suspense_scope.sent.get());
                    // Make sure parent is sent first.
                    create_effect(move || {
                        if !suspense_scope.sent.get() && suspense_scope.parent.as_ref().map_or(true, |parent| parent.get().sent.get()) {
                            let view = std::mem::take(&mut view);
                            let fragment = SuspenseFragment::new(key, view! { Show(when=true) { (view) } });
                            let mut state = state.clone();
                            sycamore_futures::spawn_local_scoped(async move {
                                let _ = state.sender.send(fragment).await;
                            });
                            suspense_scope.sent.set(true);
                        }
                    });
                });

                // Add some marker nodes so that we know start and finish of fallback.
                let start = view! { suspense-start(data-key=key.to_string()) };
                let marker = View::from(move || SsrNode::SuspenseMarker { key: key.into() });
                let end = view! { NoHydrate { suspense-end(data-key=key.to_string()) } };

                if mode == SsrMode::Blocking {
                    view! {
                        NoSsr {}
                        (start)
                        (marker)
                        (end)
                    }
                } else if mode == SsrMode::Streaming {
                    view! {
                        NoSsr {}
                        (start)
                        (marker)
                        NoHydrate(children=Children::new(fallback))
                        (end)
                    }
                } else {
                    unreachable!()
                }
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
                let (view, suspense_scope) = create_suspense_scope(move || children.call());
                let is_loading = suspense_scope.is_loading();

                create_effect(move || {
                    set_is_loading(is_loading.get());
                });

                view! {
                    Show(when=is_loading) {
                        (fallback())
                    }
                    Show(when=move || !is_loading.get()) {
                        (view)
                    }
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

                let (view, suspense_scope) = HydrationRegistry::in_suspense_scope(key, move || create_suspense_scope(move || children.call()));
                let is_loading = suspense_scope.is_loading();

                create_effect(move || set_is_loading(is_loading.get()));

                view! {
                    NoSsr {
                        Show(when=move || is_loading.get()) {
                            (fallback())
                        }
                    }
                    Show(when=move || !is_loading.get()) {
                        (view)
                    }
                }
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
                create_suspense_task(async move {
                    view.set(f().await);
                });
                ret
            }
            SsrMode::Blocking | SsrMode::Streaming => {
                // TODO: This does not properly hydrate dynamic text nodes.
                create_suspense_task(async move { f().await; });
                view! {}
            }
        }
    }
    is_ssr! {
        use std::sync::{Arc, Mutex};

        let node = Arc::new(Mutex::new(View::default()));
        create_suspense_task({
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

/// `Transition` is like [`Suspense`] except that it keeps the previous content visible until the
/// new content is ready.
#[component]
pub fn Transition(props: SuspenseProps) -> View {
    /// Only trigger outer suspense on initial render. In subsequent renders, capture the suspense
    /// scope.
    #[component(inline_props)]
    fn TransitionInner(children: Children, set_is_loading: Box<dyn FnMut(bool)>) -> View {
        // TODO: Workaround for https://github.com/sycamore-rs/sycamore/issues/718.
        let mut set_is_loading = set_is_loading;

        // We create a detatched suspense scope here to not create a deadlock with the outer
        // suspense.
        let (children, scope) = create_detatched_suspense_scope(move || children.call());
        // Trigger the outer suspense scope. Note that this is only triggered on the initial render
        // and future renders will be captured by the inner suspense scope.
        create_suspense_task(scope.until_finished());

        let is_loading = scope.is_loading();
        create_effect(move || {
            set_is_loading(is_loading.get());
        });

        view! {
            (children)
        }
    }

    view! {
        Suspense(fallback=props.fallback, children=Children::new(move || {
            view! { TransitionInner(children=props.children, set_is_loading=props.set_is_loading) }
        }))
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

use std::collections::HashMap;
use std::future::Future;

use futures::channel::mpsc::{channel, Sender};
use sycamore_futures::provide_executor_scope;

use super::*;

/// The mode in which SSR is being run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsrMode {
    /// Synchronous mode.
    ///
    /// When a suspense boundary is hit, only the fallback is rendered.
    Sync,
    /// Blocking mode.
    ///
    /// When a suspense boundary is hit, rendering is paused until the suspense is resolved.
    Blocking,
    /// Streaming mode.
    ///
    /// When a suspense boundary is hit, the fallback is rendered. Once the suspense is resolved,
    /// the rendered HTML is streamed to the client.
    Streaming,
}

pub(crate) struct SuspenseFragment {
    pub key: u32,
    pub view: View,
    pub suspend: Box<dyn Future<Output = ()>>,
}

#[derive(Clone)]
pub(crate) struct SuspenseState {
    pub sender: Sender<SuspenseFragment>,
}

/// Render a [`View`] into a static [`String`]. Useful for rendering to a string on the server side.
#[must_use]
pub fn render_to_string(view: impl FnOnce() -> View) -> String {
    is_not_ssr! {
        let _ = view;
        panic!("`render_to_string` only available in SSR mode");
    }
    is_ssr! {
        use std::cell::LazyCell;

        thread_local! {
            /// Use a static variable here so that we can reuse the same root for multiple calls to
            /// this function.
            static SSR_ROOT: LazyCell<RootHandle> = LazyCell::new(|| create_root(|| {}));
        }
        let mut buf = String::new();
        SSR_ROOT.with(|root| {
            root.dispose();
            root.run_in(|| {
                // We run this in a new scope so that we can dispose everything after we render it.
                provide_context(HydrationRegistry::new());
                provide_context(SsrMode::Sync);

                IS_HYDRATING.set(true);
                let view = view();
                IS_HYDRATING.set(false);
                ssr_node::render_recursive_view(&view, &mut buf);
            });
        });
        buf
    }
}

/// Renders a [`View`] into a static [`String`] while awaiting for all suspense boundaries to
/// resolve. Useful for rendering to a string on the server side.
#[must_use]
pub async fn render_to_string_await_suspense(view: impl FnOnce() -> View) -> String {
    is_not_ssr! {
        let _ = view;
        panic!("`render_to_string` only available in SSR mode");
    }
    is_ssr! {
        use std::cell::LazyCell;
        use std::fmt::Write;
        use futures::StreamExt;

        thread_local! {
            /// Use a static variable here so that we can reuse the same root for multiple calls to
            /// this function.
            static SSR_ROOT: LazyCell<RootHandle> = LazyCell::new(|| create_root(|| {}));
        }
        IS_HYDRATING.set(true);
        provide_executor_scope(async {
            let mut buf = String::new();

            let (sender, mut receiver) = channel(1);
            SSR_ROOT.with(|root| {
                root.dispose();
                root.run_in(|| {
                    // We run this in a new scope so that we can dispose everything after we render it.
                    provide_context(HydrationRegistry::new());
                    provide_context(SsrMode::Blocking);
                    let suspense_state = SuspenseState { sender };

                    provide_context(suspense_state);

                    let view = view();
                    ssr_node::render_recursive_view(&view, &mut buf);
                });
            });

            // Split at suspense fragment locations.
            let split = buf.split("<!--sycamore-suspense-").collect::<Vec<_>>();
            // Calculate the number of suspense fragments.
            let n = split.len() - 1;

            // Now we wait until all suspense fragments are resolved.
            let mut fragment_map = HashMap::new();
            let mut fragment_futures = Vec::new();
            let mut i = 0;
            while let Some(fragment) = receiver.next().await {
                fragment_map.insert(fragment.key, fragment.view);
                fragment_futures.push(Box::into_pin(fragment.suspend));
                i += 1;
                if i == n {
                    // We have received all suspense fragments so we shouldn't need the receiver anymore.
                    receiver.close();
                }
            }
            futures::future::join_all(fragment_futures).await;
            IS_HYDRATING.set(false);

            // Finally, replace all suspense marker nodes with rendered values.
            if let [first, rest @ ..] = split.as_slice() {
                rest.iter().fold(first.to_string(), |mut acc, s| {
                    // Try to parse the key.
                    let (num, rest) = s.split_once("-->").expect("end of suspense marker not found");
                    let key: u32 = num.parse().expect("could not parse suspense key");
                    let fragment = fragment_map.get(&key).expect("fragment not found");
                    ssr_node::render_recursive_view(fragment, &mut acc);

                    write!(&mut acc, "{rest}").unwrap();
                    acc
                })
            } else {
                unreachable!("split should always have at least one element")
            }
        }).await
    }
}

#[cfg_ssr]
fn render_suspense_fragment(fragment: SuspenseFragment) -> String {
    use std::fmt::Write;

    let mut buf = String::new();
    write!(
        &mut buf,
        "<template id=\"sycamore-suspense-{}\">",
        fragment.key,
    )
    .unwrap();
    ssr_node::render_recursive_view(&fragment.view, &mut buf);
    write!(&mut buf, "</template>").unwrap();
    buf
}

#[cfg(test)]
#[cfg(feature = "suspense")]
mod tests {
    use futures::channel::oneshot;

    use super::*;

    #[component(inline_props)]
    async fn AsyncComponent(receiver: oneshot::Receiver<()>) -> View {
        receiver.await.unwrap();
        view! {
            "Hello, async!"
        }
    }

    #[component(inline_props)]
    fn App(receiver: oneshot::Receiver<()>) -> View {
        view! {
            Suspense(fallback="fallback".into()) {
                AsyncComponent(receiver=receiver)
            }
        }
    }

    #[test]
    fn render_to_string_renders_fallback() {
        let (sender, receiver) = oneshot::channel();
        let res = render_to_string(move || view! { App(receiver=receiver) });
        assert_eq!(res, "<!--/-->fallback<!--/-->");
        assert!(sender.send(()).is_err(), "receiver should be dropped");
    }

    #[tokio::test]
    async fn render_to_string_await_suspense_works() {
        let (sender, receiver) = oneshot::channel();
        let ssr = render_to_string_await_suspense(move || view! { App(receiver=receiver) });
        futures::pin_mut!(ssr);
        assert!(futures::poll!(&mut ssr).is_pending());

        sender.send(()).unwrap();
        let res = ssr.await;
        assert_eq!(res, "<!--/--><!--/-->Hello, async!<!--/--><!--/-->");
    }
}

use super::*;

/// The mode in which SSR is being run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsrMode {
    /// Synchronous mode.
    ///
    /// When a suspense boundary is hit, only the fallback is rendered.
    Sync,
    /// Streaming mode.
    ///
    /// When a suspense boundary is hit, the fallback is never rendered. Instead, a special SSR
    /// Suspense node is created that contains a future resolving to the async content.
    Streaming,
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
                let handle = create_child_scope(|| {
                    // We run this in a new scope so that we can dispose everything after we render it.
                    provide_context(HydrationRegistry::new());
                    provide_context(SsrMode::Sync);

                    IS_HYDRATING.set(true);
                    let view = view();
                    IS_HYDRATING.set(false);
                    for node in view.nodes {
                        ssr_node::render_recursive(node, &mut buf);
                    }
                });
                handle.dispose();
            });
        });
        buf
    }
}

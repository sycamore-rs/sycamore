//! Web support for Sycamore.

#![allow(unused_imports)]
pub mod portal;

use sycamore_core::component::Children;
use sycamore_macro::{component, Props};
use sycamore_reactive::Scope;
use sycamore_web::hydrate::HydrationState;
use sycamore_web::render::RenderEnv;
use sycamore_web::web_node::WebNode;
/// Re-export sycamore-web
pub use sycamore_web::*;

use crate::prelude::*;

/// A macro for ergonomically creating complex UI layouts in HTML.
///
/// To learn more about the view syntax, see [the chapter on views](https://sycamore-rs.netlify.app/docs/basics/view)
/// in the Sycamore Book.
#[macro_export]
macro_rules! html_view {
    ($($t:tt)*) => {
        $crate::view_with_elements!($crate::web::html, $($t)*)
    };
}

/// Like [`view!`] but only creates a single raw node instead.
///
/// # Example
///
/// ```
/// use sycamore::prelude::*;
///
/// #[component]
/// pub fn MyComponent<G: Html>(cx: Scope) -> View<G> {
///     let cool_button: G = node! { cx, button { "The coolest 😎" } };
///
///     cool_button.set_property("myProperty", &"Epic!".into());
///
///     View::new_node(cool_button)
/// }
/// ```
#[macro_export]
macro_rules! html_node {
    ($($t:tt)*) => {
        $crate::node_with_elements!($crate::web::html, $($t)*)
    };
}

#[doc(hidden)]
pub mod macros {
    pub use crate::{html_node as node, html_view as view};
}

/// Props for [`NoHydrate`].
#[cfg(feature = "hydrate")]
#[derive(Props, Debug)]
pub struct NoHydrateProps<'a> {
    children: Children<'a, WebNode>,
}

/// Render the children of this component in a scope that will not be hydrated.
///
/// When using `SsrNode`, this means that hydration markers won't be generated. When using
/// `HydrateNode`, this means that the entire sub-tree will be ignored. When using `DomNode`,
/// rendering proceeds as normal.
///
/// The children are wrapped inside a `<div>` element to prevent conflicts with surrounding
/// elements.
#[cfg(feature = "hydrate")]
#[component]
pub fn NoHydrate<'a>(cx: Scope<'a>, props: NoHydrateProps<'a>) -> View {
    use sycamore_core::render::insert;
    use sycamore_web::hydrate::{is_hydrating, without_hydration_state};
    use sycamore_web::render::{get_render_env, RenderEnv};

    let node_ref = create_node_ref(cx);
    let view = view! { cx,
        div(_ref=node_ref)
    };
    if is_hydrating(cx) {
        match get_render_env(cx) {
            RenderEnv::Dom => {
                // We don't want to hydrate the children, so we just do nothing.
            }
            RenderEnv::Ssr => {
                // If we have a hydration context, remove it in this scope so that hydration markers
                // are not generated
                let nodes = without_hydration_state(cx, || props.children.call(cx));
                insert(cx, &node_ref.get(), nodes, None, None, false);
            }
        }
    } else {
        // Just continue rendering as normal.
        let nodes = props.children.call(cx);
        insert(cx, &node_ref.get(), nodes, None, None, false);
    }
    view
}

/// Props for [`NoSsr`].
#[cfg(feature = "hydrate")]
#[derive(Props, Debug)]
pub struct NoSsrProps<'a> {
    children: Children<'a, WebNode>,
}

/// Only render the children of this component in the browser.
/// The children are wrapped inside a `<div>` element to prevent conflicts with surrounding
/// elements.
#[cfg(feature = "hydrate")]
#[component]
pub fn NoSsr<'a>(cx: Scope<'a>, props: NoSsrProps<'a>) -> View {
    use sycamore_web::hydrate::without_hydration_state;
    use sycamore_web::render::{get_render_env, RenderEnv};

    without_hydration_state(cx, || {
        let node = match get_render_env(cx) {
            RenderEnv::Dom => props.children.call(cx),
            // We don't want to render the children, so we just do nothing.
            RenderEnv::Ssr => view! { cx, },
        };

        view! { cx,
            div { (node) }
        }
    })
}

/// Render a [`View`] into a static [`String`]. Useful for rendering to a string on the server side.
///
/// Waits for suspense to be loaded before returning.
#[cfg(all(feature = "ssr", feature = "suspense"))]
pub async fn render_to_string_await_suspense(
    f: impl FnOnce(Scope<'_>) -> View + 'static,
) -> String {
    use futures::channel::oneshot;
    use sycamore_futures::spawn_local_scoped;

    let (sender, receiver) = oneshot::channel();

    let disposer = create_scope(|cx| {
        spawn_local_scoped(cx, async move {
            let ssr = render_to_string_await_suspense_with_scope(cx, f).await;
            sender
                .send(ssr)
                .expect("receiving end should not be dropped");
        });
    });

    let ssr = receiver.await.expect("rendering should complete");

    // SAFETY: we are done with the scope now.
    unsafe {
        disposer.dispose();
    }
    ssr
}

/// Same as [`render_to_string_await_suspense`] but with a pre-created scope.
#[cfg(all(feature = "ssr", feature = "suspense"))]
pub async fn render_to_string_await_suspense_with_scope(
    cx: Scope<'_>,
    f: impl FnOnce(Scope<'_>) -> View,
) -> String {
    use sycamore_web::web_node::ssr::WriteToString;

    // Provide the environment context.
    provide_context(cx, RenderEnv::Ssr);
    provide_context(cx, HydrationState::new());

    let mut buf = String::new();

    let v = crate::suspense::await_suspense(cx, async { f(cx) }).await;
    for node in v.flatten() {
        node.as_ssr_node()
            .expect("expected SSR node")
            .write_to_string(&mut buf);
    }

    buf
}

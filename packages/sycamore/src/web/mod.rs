//! Web support for Sycamore.

pub mod html;
pub mod portal;

pub use sycamore_web::*;

#[allow(unused_imports)]
use crate::prelude::*;

/// Render a [`View`] into a static [`String`]. Useful
/// for rendering to a string on the server side.
///
/// Waits for suspense to be loaded before returning.
///
/// _This API requires the following crate features to be activated: `suspense`, `ssr`_
#[cfg(all(feature = "ssr", feature = "suspense"))]
pub async fn render_to_string_await_suspense(
    view: impl FnOnce() -> View<SsrNode> + 'static,
) -> String {
    use std::cell::RefCell;
    use std::rc::Rc;

    use futures::channel::oneshot;
    use sycamore_futures::spawn_local_scoped;

    use crate::utils::hydrate::with_hydration_context_async;

    let mut ret = String::new();
    let v = Rc::new(RefCell::new(None));
    let (sender, receiver) = oneshot::channel();
    let disposer = create_root({
        let v = Rc::clone(&v);
        move || {
            spawn_local_scoped(async move {
                *v.borrow_mut() = Some(
                    with_hydration_context_async(async {
                        crate::suspense::await_suspense(async { view() }).await
                    })
                    .await,
                );
                sender
                    .send(())
                    .expect("receiving end should not be dropped");
            });
        }
    });
    receiver.await.expect("rendering should complete");
    let v = v.borrow().clone().unwrap();
    for node in v.flatten() {
        node.write_to_string(&mut ret);
    }

    disposer.dispose();

    ret
}

/// Props for [`NoHydrate`].
#[cfg(feature = "hydrate")]
#[derive(Props, Debug)]
pub struct NoHydrateProps<G: GenericNode> {
    children: Children<G>,
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
pub fn NoHydrate<G: Html>(props: NoHydrateProps<G>) -> View<G> {
    use crate::utils::{hydrate, render};

    let node_ref = create_node_ref();
    let v = view! {
        div(ref=node_ref) {}
    };
    if G::CLIENT_SIDE_HYDRATION && !hydrate::hydration_completed() {
        // We don't want to hydrate the children, so we just do nothing.
    } else if G::USE_HYDRATION_CONTEXT {
        // If we have a hydration context, remove it in this scope so that hydration markers are not
        // generated.
        let nodes = hydrate::with_no_hydration_context(|| props.children.call());
        render::insert(&node_ref.get_raw(), nodes, None, None, false);
    } else {
        // Just continue rendering as normal.
        let nodes = props.children.call();
        render::insert(&node_ref.get_raw(), nodes, None, None, false);
    };
    v
}

/// Props for [`NoSsr`].
#[cfg(feature = "hydrate")]
#[derive(Props, Debug)]
pub struct NoSsrProps<G: GenericNode> {
    children: Children<G>,
}

/// Only render the children of this component in the browser.
/// The children are wrapped inside a `<div>` element to prevent conflicts with surrounding
/// elements.
#[cfg(feature = "hydrate")]
#[component]
pub fn NoSsr<G: Html>(props: NoSsrProps<G>) -> View<G> {
    use crate::utils::hydrate;

    let node = if !G::IS_BROWSER {
        // We don't want to render the children, so we just do nothing.
        view! {}
    } else if G::USE_HYDRATION_CONTEXT {
        // Since the nodes were not rendered on the server, there is nothing to hydrate.
        hydrate::with_no_hydration_context(|| props.children.call())
    } else {
        // Just continue rendering as normal.
        props.children.call()
    };
    view! {
        div { (node) }
    }
}

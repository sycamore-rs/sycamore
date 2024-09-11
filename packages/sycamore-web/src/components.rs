//! Definition for the [`NoSsr`] and [`NoHydrate`] components.

use sycamore_macro::{component, view};

use crate::*;

/// Component that is only renders its children on the client side.
///
/// This is useful when wrapping parts of your app that are not intended to be server-side
/// rendered, e.g. highly interactive components such as graphs, etc...
#[component(inline_props)]
pub fn NoSsr(children: Children) -> View {
    if is_ssr!() {
        view! { no-ssr() }
    } else {
        let marker = create_node_ref();
        let view = view! { no-ssr(r#ref=marker) };
        on_mount(move || {
            let marker = marker.get();
            let parent = marker.parent_node().unwrap();

            let children = children.call();
            for node in children.as_web_sys() {
                parent.insert_before(&node, Some(&marker)).unwrap();
            }
            parent.remove_child(&marker).unwrap();
        });
        view
    }
}

/// Components that do not need, or should not be hydrated on the client side.
///
/// This is useful when large parts of your app do not require client-side interactivity such as
/// static content.
///
/// However, this component will still be rendered on the client side if it is created after the
/// initial hydration phase is over, e.g. navigating to a new page with a `NoHydrate` component.
#[component(inline_props)]
pub fn NoHydrate(children: Children) -> View {
    if is_ssr!() {
        let is_hydrating = IS_HYDRATING.replace(false);
        let children = children.call();
        IS_HYDRATING.set(is_hydrating);
        children
    } else if IS_HYDRATING.get() {
        view! {}
    } else {
        children.call()
    }
}

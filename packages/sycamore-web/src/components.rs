//! Definition for the [`NoSsr`] and [`NoHydrate`] components.

use sycamore_macro::{component, view};

use crate::*;

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

#[component(inline_props)]
pub fn NoHydrate(children: Children) -> View {
    if is_ssr!() {
        let is_hydrating = IS_HYDRATING.replace(false);
        let children = children.call();
        IS_HYDRATING.set(is_hydrating);
        children
    } else {
        view! {}
    }
}

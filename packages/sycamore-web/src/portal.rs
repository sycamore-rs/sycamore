use sycamore_macro::component;

use crate::*;

/// A portal into a different part of the DOM. Only renders in client side rendering (CSR) mode.
/// Does nothing in SSR mode.
#[component(inline_props)]
pub fn Portal<'a, T: Into<View> + Default>(selector: &'a str, children: T) -> View {
    if is_not_ssr!() {
        let Some(parent) = document().query_selector(selector).unwrap() else {
            panic!("element matching selector `{selector}` not found");
        };

        let children = children.into();
        let nodes = children.as_web_sys();

        for node in &nodes {
            parent.append_child(node).unwrap();
        }

        on_cleanup(move || {
            // FIXME: we should be using start and end nodes to remove dynamic elements.
            for node in nodes {
                parent.remove_child(&node).unwrap();
            }
        });
    }
    View::default()
}

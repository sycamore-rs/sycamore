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

        let start = HtmlNode::create_marker_node();
        let start_node = start.as_web_sys().clone();
        let end = HtmlNode::create_marker_node();
        let end_node = end.as_web_sys().clone();
        let children: View = (start, children.into(), end).into();

        let nodes = children.as_web_sys();
        for node in &nodes {
            parent.append_child(node).unwrap();
        }

        on_cleanup(move || {
            let nodes = get_nodes_between(&start_node, &end_node);
            for node in nodes {
                parent.remove_child(&node).unwrap();
            }
            parent.remove_child(&start_node).unwrap();
            parent.remove_child(&end_node).unwrap();
        });
    }
    View::default()
}

use sycamore_macro::component;

use crate::*;

/// A portal into a different part of the DOM. Only renders in client side rendering (CSR) mode.
/// Does nothing in SSR mode.
#[component(inline_props)]
pub fn Portal<'a, T: Into<View> + Default>(selector: &'a str, children: T) -> View {
    if is_not_ssr!() {
        //let Some(parent) = document().query_selector(selector).unwrap() else {
        //    panic!("element matching selector `{selector}` not found");
        //};
        //
        //let children = children.into();
        //let nodes = children.as_web_sys();
        //
        //DomRenderer.render(&parent, children);
        //
        //on_cleanup(move || {
        //    for node in nodes {
        //        let node = node.get().unwrap();
        //        parent.remove_child(node).unwrap();
        //    }
        //});
        todo!("portals")
    }
    View::default()
}

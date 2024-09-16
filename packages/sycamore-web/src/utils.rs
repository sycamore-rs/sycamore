//! Utility functions. Intended for internal use only.

use crate::*;

/// Get all nodes between `start` and `end`.
///
/// If `end` is before `start`, all nodes after `start` will be returned.
///
/// The range is exclusive so `start` and `end` will not be included.
#[must_use]
pub fn get_nodes_between(start: &web_sys::Node, end: &web_sys::Node) -> Vec<web_sys::Node> {
    let parent = start.parent_node().unwrap();
    debug_assert_eq!(
        parent,
        end.parent_node().unwrap(),
        "parents of `start` and `end` do not match"
    );

    let mut nodes = Vec::new();

    let mut next = start.next_sibling();
    while let Some(current) = next {
        let tmp = current.next_sibling();
        if &current == end {
            break;
        } else {
            nodes.push(current);
        }
        next = tmp;
    }

    nodes
}

/// Wrap all the nodes in a [`View`] in a document fragment.
///
/// This is useful when the view is dynamically changed without being mounted since this will not
/// update the DOM. Wrapping the nodes in a document fragment will allow you to dynamically update
/// the view while it is detatched, and then insert it into the DOM later.
///
/// This only works on the client side.
pub fn wrap_in_document_fragment(view: View) -> View {
    let fragment = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_document_fragment();

    for node in view.as_web_sys() {
        fragment.append_child(&node).unwrap();
    }

    View::from_node(HtmlNode::from_web_sys(fragment.into()))
}

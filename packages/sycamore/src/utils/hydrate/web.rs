//! Utilities for client-side hydration in the browser.

use wasm_bindgen::JsCast;
use web_sys::{window, Comment, Element, Node};

use super::*;
use crate::generic_node::HydrateNode;
use crate::view::View;

const COMMENT_NODE_TYPE: u16 = 8;

/// Gets the element with the next hydration-key or `None` if not found.
/// This method basically queries elements with the `data-hk` attribute.
pub fn get_next_element() -> Option<Element> {
    if let Some(hk) = get_next_id() {
        window()
            .unwrap()
            .document()
            .unwrap()
            .query_selector(&format!("[data-hk=\"{}.{}\"]", hk.0, hk.1))
            .unwrap()
    } else {
        None
    }
}

/// Gets the next node surrounded by `<!#>` and `<!/>`. Removes the start node so that next call
/// will return next marked nodes.
pub fn get_next_marker(parent: &Node) -> Option<View<HydrateNode>> {
    if hydration_completed() {
        None
    } else {
        // A Vec of nodes that are between the start and end markers.
        let mut buf: Vec<View<HydrateNode>> = Vec::new();
        // `true` if between start and end markers. Nodes that are visited when start is true are
        // added to buf.
        let mut start = false;
        let mut start_marker = None;
        // Iterate through the children of parent.
        let children = parent.child_nodes();
        for child in (0..children.length()).filter_map(|i| children.get(i)) {
            if child.node_type() == COMMENT_NODE_TYPE {
                let v = child.node_value();
                if v.as_deref() == Some("#") {
                    start = true; // Start hydration marker.
                    start_marker = Some(child);

                    // NOTE: we can't delete the start node now because that would mess up with the
                    // node indexes.
                } else if v.as_deref() == Some("/") {
                    if start {
                        // Delete start marker. This will ensure that the next time this function is
                        // called, the same span of nodes will not be returned.
                        start_marker.unwrap().unchecked_into::<Comment>().remove();

                        // End of node span. Return accumulated nodes in buf.
                        return Some(View::new_fragment(buf));
                    } else {
                        // Still inside a span. Continue.
                        buf.push(View::new_node(HydrateNode::from_web_sys(child)));
                    }
                }
            } else if start {
                buf.push(View::new_node(HydrateNode::from_web_sys(child)));
            }
        }

        None
    }
}

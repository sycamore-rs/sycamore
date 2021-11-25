//! Utilities for client-side hydration in the browser.

use wasm_bindgen::JsCast;
use web_sys::{window, Comment, Element, Node};

use crate::view::View;
use crate::HydrateNode;

use super::*;

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
        let mut buf = Vec::new();
        let mut start = false;
        let children = parent.child_nodes();
        for i in 0..children.length() {
            let child = children.get(i);
            if let Some(child) = child {
                if child.node_type() == COMMENT_NODE_TYPE {
                    let v = child.node_value();
                    if v.as_deref() == Some("#") {
                        start = true; // Start of hydration marker.

                        // Delete start marker.
                        child.unchecked_into::<Comment>().remove();
                    } else if v.as_deref() == Some("/") {
                        if start {
                            return Some(View::new_fragment(buf)); // End of hydration marker.
                        } else {
                            // Not a hydration marker.
                            buf.push(View::new_node(HydrateNode::from_web_sys(child)));
                        }
                    }
                } else {
                    buf.push(View::new_node(HydrateNode::from_web_sys(child)));
                }
            }
        }

        None
    }
}

//! Utilities for client-side hydration in the browser.

use web_sys::{window, Element, Node};

use super::*;

const COMMENT_NODE_TYPE: u16 = 8;

/// Gets the element with the next hydration-key or `None` if not found.
/// This method basically queries elements with the `data-hk` attribute.
pub fn get_next_element() -> Option<Element> {
    if hydration_completed() {
        None
    } else {
        let id = get_next_id();
        window()
            .unwrap()
            .document()
            .unwrap()
            .query_selector(&format!("[data-hk=\"{}.{}\"]", id.0, id.1))
            .unwrap()
    }
}

pub fn get_next_marker(parent: &Node) -> Option<Vec<Node>> {
    if hydration_completed() {
        None
    } else {
        let mut buf = Vec::new();
        let mut start = false;
        let children = parent.child_nodes();
        for i in 0..children.length() {
            let child = children.get(i).unwrap();
            if child.node_type() == COMMENT_NODE_TYPE {
                let v = child.node_value();
                if v.as_deref() == Some("#") {
                    start = true; // Start of hydration marker.
                } else if v.as_deref() == Some("/") {
                    if start {
                        return Some(buf); // End of hydration marker.
                    } else {
                        buf.push(child); // Not a hydration marker.
                    }
                }
            } else {
                buf.push(child);
            }
        }

        None
    }
}

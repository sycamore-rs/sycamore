//! Hydration utilities for Sycamore.

use std::cell::Cell;
use std::fmt;

use sycamore_reactive::*;
use wasm_bindgen::JsCast;
use web_sys::{Comment, Element, HtmlElement, Node};

/// A hydration key. This is used to identify a dynamic node that needs to be hydrated.
///
/// This is represented as a pair of `u32`s. The first `u32` is the component id and the second
/// `u32` is the element id. Each time a component is rendered, the component id is incremented, and
/// each time an element is created, the element id is incremented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HydrationKey(u32, u32);

impl HydrationKey {
    /// Get the component id.
    pub fn component_id(self) -> u32 {
        self.0
    }

    /// Get the element id.
    pub fn element_id(self) -> u32 {
        self.1
    }

    /// Create a blank, invalid, hydration key.
    pub fn null() -> Self {
        Self(0, 0)
    }
}

impl fmt::Display for HydrationKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

/// The current hydration state.
/// This should be a Sycamore context at the root scope in both client and SSR mode.
#[derive(Debug)]
pub struct HydrationState {
    current_component_id: Cell<u32>,
    max_component_id: Cell<u32>,
    current_element_id: Cell<u32>,
    /// If this is `false`, then the app should behave as if it is not being hydrated.
    active: Cell<bool>,
}

impl HydrationState {
    pub fn with_no_hydration_state<T>(cx: Scope, f: impl FnOnce() -> T) -> T {
        let mut h_state = try_use_context::<Self>(cx);
        if let Some(h_state) = &mut h_state {
            let old_active = h_state.active.get();
            h_state.active.set(false);
            let ret = f();
            h_state.active.set(old_active);
            ret
        } else {
            f()
        }
    }

    /// Increments the element id and returns a new hydration key.
    #[must_use]
    pub fn next_key(&self) -> HydrationKey {
        self.current_element_id
            .set(self.current_element_id.get() + 1);
        HydrationKey(
            self.current_component_id.get(),
            self.current_element_id.get(),
        )
    }

    /// Get the current hydration key.
    pub fn current_key(&self) -> HydrationKey {
        HydrationKey(
            self.current_component_id.get(),
            self.current_element_id.get(),
        )
    }

    /// Increments the component id and resets the element id. Runs the provided closure and
    /// restores the previous state.
    pub fn enter_component<T>(&self, f: impl FnOnce() -> T) -> T {
        let old_component_id = self.current_component_id.get();
        let old_element_id = self.current_element_id.get();

        self.current_component_id
            .set(self.max_component_id.get() + 1);
        self.max_component_id.set(self.max_component_id.get() + 1);
        self.current_element_id.set(0);

        let ret = f();

        self.current_component_id.set(old_component_id);
        self.current_element_id.set(old_element_id);
        ret
    }

    /// Create a new default hydration state.
    pub fn new() -> Self {
        Self {
            current_component_id: 0.into(),
            max_component_id: 0.into(),
            current_element_id: 0.into(),
            active: true.into(),
        }
    }
}

/// The hydration context. This collects the root node and all dynamic nodes that need to be
/// hydrated so that they can be accessed later.
#[derive(Debug)]
pub struct HydrationCtx {
    /// The root element where the app is mounted.
    pub root: Element,
    /// The dynamic elements that need to be hydrated.
    /// These elements are indexed by their hydration key.
    pub els: hashbrown::HashMap<HydrationKey, Node>,
}

impl HydrationCtx {
    // Uses query_selector_all to get all elements with the `data-hk` attribute under the `root` and
    // returns a new `HydrateCtx`.
    pub fn new_from_root(root: Element) -> Self {
        let mut els = hashbrown::HashMap::new();
        let query = root.query_selector_all("[data-hk]").unwrap();

        for i in 0..query.length() {
            let el = query.get(i).unwrap();
            let hk = el
                .unchecked_ref::<Element>()
                .get_attribute("data-hk")
                .unwrap();
            // Parse the hydration key. If malformed, skip this element.
            let (before, after) = hk.split_once('.').unwrap();
            let Ok(component_id) = before.parse() else {
                continue;
            };
            let Ok(view_id) = after.parse() else {
                continue;
            };
            let hk = HydrationKey(component_id, view_id);
            els.insert(hk, el);
        }

        Self { root, els }
    }

    /// Get an element by its hydration key.
    pub fn get_element_by_key(&self, hk: HydrationKey) -> Option<&Node> {
        self.els.get(&hk)
    }
}

/// Gets the next node surrounded by `<!#>` and `<!/>`. Removes the start node so that next call
/// will return next marked nodes.
pub fn get_next_markers(el: &HtmlElement) -> Option<Vec<Node>> {
    // A Vec of nodes that are between the start and end markers.
    let mut buf: Vec<Node> = Vec::new();
    // `true` if between start and end markers. Nodes that are visited when start is true are
    // added to buf.
    let mut start = false;
    let mut start_marker = None;
    // Iterate through the children of parent.
    let children = el.child_nodes();
    for child in (0..children.length()).filter_map(|i| children.get(i)) {
        if child.node_type() == Node::COMMENT_NODE {
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
                    return Some(buf);
                } else {
                    // Still inside a span. Continue.
                    buf.push(child);
                }
            }
        } else if start {
            buf.push(child);
        }
    }
    None
}

/// Retrieves the [`HydrationState`] from the [`Scope`] context.
///
/// # Panics
///
/// This will panic if the [`HydrationState`] is not in the context.
pub(crate) fn use_hydration_state(cx: Scope) -> &HydrationState {
    use_context::<HydrationState>(cx)
}

/// Retrieves the [`HydrateCtx`] from the [`Scope`] context.
///
/// # Panics
///
/// This will panic if the [`HydrateCtx`] is not in the context.
pub(crate) fn use_hydration_ctx(cx: Scope) -> &HydrationCtx {
    use_context::<HydrationCtx>(cx)
}

/// Returns `true` if the app is being hydrated.
/// Note that this will return `true` on the server as well, since the hydration machinery is used
/// there.
pub fn is_hydrating(cx: Scope) -> bool {
    if let Some(h_state) = try_use_context::<HydrationState>(cx) {
        h_state.active.get()
    } else {
        false
    }
}

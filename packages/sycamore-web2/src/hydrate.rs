//! Hydration utilities for Sycamore.

use std::cell::Cell;

use sycamore_reactive::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node};

/// A hydration key. This is used to identify a dynamic node that needs to be hydrated.
///
/// This is represented as a pair of `u32`s. The first `u32` is the component id and the second
/// `u32` is the view id. Each time a component is rendered, the component id is incremented, and
/// each time a view is created (using `view!`), the view id is incremented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HydrationKey(u32, u32);

impl HydrationKey {
    /// Get the component id.
    pub fn component_id(self) -> u32 {
        self.0
    }

    /// Get the view id.
    pub fn view_id(self) -> u32 {
        self.1
    }
}

/// The current hydration state.
/// This should be a Sycamore context at the root scope in both client and SSR mode.
#[derive(Debug)]
pub struct HydrationState {
    current_component_id: Cell<u32>,
    max_component_id: Cell<u32>,
    current_view_id: Cell<u32>,
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

    /// Increments the view id and returns a new hydration key.
    pub fn increment_view_id(&self) -> HydrationKey {
        let hk = HydrationKey(self.current_component_id.get(), self.current_view_id.get());
        self.current_view_id.set(self.current_view_id.get() + 1);
        hk
    }

    /// Increments the component id and resets the view id.
    /// Runs the provided closure and restores the previous state.
    pub fn increment_component_id<T>(&self, f: impl FnOnce() -> T) -> T {
        let old_component_id = self.current_component_id.get();
        let old_view_id = self.current_view_id.get();

        self.current_component_id
            .set(self.max_component_id.get() + 1);
        self.max_component_id.set(self.max_component_id.get() + 1);
        self.current_view_id.set(0);

        let ret = f();

        self.current_component_id.set(old_component_id);
        self.current_view_id.set(old_view_id);
        ret
    }

    /// Create a new default hydration state.
    pub fn new() -> Self {
        Self {
            current_component_id: 0.into(),
            max_component_id: 0.into(),
            current_view_id: 0.into(),
            active: true.into(),
        }
    }
}

/// The hydration context. This collects the root node and all dynamic nodes that need to be
/// hydrated so that they can be accessed later.
#[derive(Debug)]
pub struct HydrateCtx {
    /// The root element where the app is mounted.
    pub root: Element,
    /// The dynamic elements that need to be hydrated.
    /// These elements are indexed by their hydration key.
    pub els: hashbrown::HashMap<HydrationKey, Node>,
}

impl HydrateCtx {
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
    pub fn get_element(&self, hk: HydrationKey) -> Option<&Node> {
        self.els.get(&hk)
    }
}

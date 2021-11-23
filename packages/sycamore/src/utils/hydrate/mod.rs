//! Hydration support for Sycamore.

pub mod web;

use std::cell::RefCell;

thread_local! {
    static HYDRATION_CONTEXT: RefCell<Option<HydrationRegistry>> = RefCell::new(None);
}

pub fn with_hydration_context<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    HYDRATION_CONTEXT.with(|context| {
        if context.borrow().is_some() {
            panic!("hydration context already exists");
        } else {
            *context.borrow_mut() = Some(HydrationRegistry::new());
            let r = f();
            *context.borrow_mut() = None;
            r
        }
    })
}

/// Returns a tuple of the current component id and the current hydration key.
/// Increments the hydration key.
///
/// If hydration context does not exist, returns `None`.
pub fn get_next_id() -> Option<(usize, usize)> {
    HYDRATION_CONTEXT.with(|context| {
        let mut context = context.borrow_mut();
        context
            .as_mut()
            .map(|reg| (reg.current_component_id, reg.get_next_id()))
    })
}

/// Returns a tuple of the current component id and the current hydration key.
pub fn get_current_id() -> Option<(usize, usize)> {
    HYDRATION_CONTEXT.with(|context| {
        let mut context = context.borrow_mut();
        context
            .as_mut()
            .map(|reg| (reg.current_component_id, reg.current_id))
    })
}

/// Returns `true` if hydration has completed.
pub fn hydration_completed() -> bool {
    HYDRATION_CONTEXT.with(|context| context.borrow().is_none())
}

/// Increments the hydration component id, calls the callback, and resets the component id to
/// previous value.
///
/// If outside of a hydration context, does nothing.
pub fn hydrate_component<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    HYDRATION_CONTEXT.with(|context| {
        if context.borrow().is_some() {
            let current_component_id = context.borrow().as_ref().unwrap().current_component_id;
            context.borrow_mut().as_mut().unwrap().current_component_id += 1;
            let r = f();
            context.borrow_mut().as_mut().unwrap().current_component_id = current_component_id;
            r
        } else {
            f()
        }
    })
}

/// A manager for the current hydration state.
pub struct HydrationRegistry {
    pub current_id: usize,
    pub current_component_id: usize,
}

impl HydrationRegistry {
    pub fn new() -> Self {
        Self {
            current_id: 0,
            current_component_id: 0,
        }
    }

    /// Gets the next id.
    pub fn get_next_id(&mut self) -> usize {
        let id = self.current_id;
        self.current_id += 1;
        id
    }
}

impl Default for HydrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

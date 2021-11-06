//! Hydration support for Sycamore.

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

pub fn get_next_id() -> usize {
    HYDRATION_CONTEXT.with(|context| {
        let mut context = context.borrow_mut();
        if let Some(context) = context.as_mut() {
            context.get_next_id()
        } else {
            panic!("hydration context does not exist");
        }
    })
}

/// Increments the hydration component id, calls the callback, and resets the component id to previous value.
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
            panic!("hydration context does not exist");
        }
    })
}

/// A manager for the current hydration state.
#[derive(Default)]
pub struct HydrationRegistry {
    pub current_id: usize,
    pub current_component_id: usize,
}

impl HydrationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the next id.
    pub fn get_next_id(&mut self) -> usize {
        let id = self.current_id;
        self.current_id += 1;
        id
    }
}

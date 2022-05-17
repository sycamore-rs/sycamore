//! Hydration support for Sycamore.
//!
//! This is backend-agnostic. If you are looking specifically for hydrating the DOM, see the
//! `sycamore-web` crate.

use std::cell::RefCell;

thread_local! {
    static HYDRATION_CONTEXT: RefCell<Option<HydrationRegistry>> = RefCell::new(None);
}

/// Run the closure inside a hydration context. If already inside a hydration context, creates a
/// nested context.
pub fn with_hydration_context<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    HYDRATION_CONTEXT.with(|context| {
        // Save previous context to restore later.
        let prev = *context.borrow();
        *context.borrow_mut() = Some(HydrationRegistry::new());
        let r = f();
        *context.borrow_mut() = prev;
        r
    })
}

/// Run the async future inside a hydration context. If already inside a hydration context, creates
/// a nested context.
///
/// Same as [`with_hydration_context`] but allows for async futures.
pub async fn with_hydration_context_async<F, R>(f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let mut prev = None;
    HYDRATION_CONTEXT.with(|context| {
        // Save previous context to restore later.
        prev = Some(*context.borrow());
        *context.borrow_mut() = Some(HydrationRegistry::new());
    });
    let r = f.await;
    HYDRATION_CONTEXT.with(|context| {
        *context.borrow_mut() = prev.unwrap();
    });
    r
}

/// Run the closure without a hydration context. If called within an hydration context, the old
/// hydration context is restored when the closure returns.
pub fn with_no_hydration_context<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    HYDRATION_CONTEXT.with(|context| {
        // Save previous context to restore later.
        let prev = *context.borrow();
        *context.borrow_mut() = None;
        let r = f();
        *context.borrow_mut() = prev;
        r
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
            let prev_id;
            let prev_component_id;
            {
                let mut context = context.borrow_mut();
                let context = context.as_mut().unwrap();
                // Store previous state to restore after component.
                prev_component_id = context.current_component_id;
                prev_id = context.current_id;

                context.current_component_id = context.next_component_id;
                context.next_component_id += 1;
                context.current_id = 0; // Reset current_id to 0.
            }
            let r = f();
            context.borrow_mut().as_mut().unwrap().current_component_id = prev_component_id;
            context.borrow_mut().as_mut().unwrap().current_id = prev_id;
            r
        } else {
            f()
        }
    })
}

/// A manager for the current hydration state.
#[derive(Debug, Clone, Copy)]
pub struct HydrationRegistry {
    /// The current node id. This is incremented every time a new element is created.
    pub current_id: usize,
    /// The current component id. This is incremented every time a new component is created.
    /// Every time this is incremented, `current_id` is reset to `0`. This is to add more tolerance
    /// to hydration so that one component that doesn't hydrate correctly will not prevent other
    /// components from hydrating.
    pub current_component_id: usize,
    /// The next component id. We need to save this because exiting the component scope decrements
    /// the current component id. This is to ensure that component ids are unique for each
    /// instance of a component.
    pub next_component_id: usize,
}

impl HydrationRegistry {
    /// Create a new [`HydrationRegistry`] with defaults.
    pub fn new() -> Self {
        Self {
            current_id: 0,
            current_component_id: 0,
            next_component_id: 1,
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

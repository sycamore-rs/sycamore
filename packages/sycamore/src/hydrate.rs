//! Hydration support for Sycamore.

/// A manager for the current hydration state.
pub struct HydrationRegistry {
    pub current_id: u32,
    pub current_component_id: u32,
}

//! Internal utilities for Sycamore.
//!
//! # Stability
//! This API is considered implementation details and should not at any time be considered stable.
//! The API can change without warning and without a semver compatible release.

use crate::generic_node::GenericNode;
use crate::prelude::*;

#[cfg(feature = "hydrate")]
pub mod hydrate;
pub use sycamore_core::render;

/// If `el` is a `HydrateNode`, use `get_next_marker` to get the initial node value.
pub fn initial_node<G: GenericNode>(_el: &G) -> Option<View<G>> {
    #[cfg(feature = "hydrate")]
    {
        use std::any::Any;
        use std::mem::ManuallyDrop;
        use std::ptr;

        if let Some(el) = <dyn Any>::downcast_ref::<HydrateNode>(_el) {
            let initial = hydrate::web::get_next_marker(&el.inner_element());
            // Do not drop the HydrateNode because it will be cast into a GenericNode.
            let initial = ManuallyDrop::new(initial);
            // SAFETY: This is safe because we already checked that the type is HydrateNode.
            // initial is wrapped inside ManuallyDrop to prevent double drop.
            unsafe { ptr::read(&initial as *const _ as *const _) }
        } else {
            None
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        None
    }
}

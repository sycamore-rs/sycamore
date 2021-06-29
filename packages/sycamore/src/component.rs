//! The definition for the [`Component`] trait.

use crate::generic_node::GenericNode;

/// Trait that is implemented by components. Should not be implemented manually. Use the
/// [`component`](sycamore_macro::component) macro instead.
pub trait Component<G: GenericNode> {
    /// The name of the component (for use in debug mode).
    const NAME: &'static str = "UnnamedComponent";
}

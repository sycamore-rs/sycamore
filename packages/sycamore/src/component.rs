//! The definition of the [`Component`] trait.

use crate::generic_node::GenericNode;
use crate::prelude::View;

/// Trait that is implemented by components. Should not be implemented manually. Use the
/// [`component`](sycamore_macro::component) macro instead.
pub trait Component<G: GenericNode> {
    /// The name of the component (for use in debug mode). In release mode, this will default to
    /// `"UnnamedComponent"`
    const NAME: &'static str = "UnnamedComponent";
    /// The type of the properties passed to the component.
    type Props;

    /// Create a new component with an instance of the properties.
    ///
    /// The double underscores (`__`) are to prevent conflicts with other trait methods. This is
    /// because we cannot use fully qualified syntax here because it prevents type inference.
    fn __create_component(props: Self::Props) -> View<G>;
}

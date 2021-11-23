//! The definition of the [`Component`] trait.

use sycamore_reactive::untrack;

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
    fn create_component(props: Self::Props) -> View<G>;
}

/// Instantiates a component.
#[inline(always)]
pub fn instantiate_component<G: GenericNode, C: Component<G>>(props: C::Props) -> View<G> {
    if G::USE_HYDRATION_CONTEXT {
        #[cfg(feature = "hydrate")]
        return crate::utils::hydrate::hydrate_component(|| untrack(|| C::create_component(props)));
        #[cfg(not(feature = "hydrate"))]
        return untrack(|| C::create_component(props));
    } else {
        untrack(|| C::create_component(props))
    }
}

/// Alias to [`instantiate_component`]. For use in proc-macro output.
///
/// The double underscores (`__`) are to prevent conflicts with other trait methods. This is
/// because we cannot use fully qualified syntax here because it prevents type inference.
#[doc(hidden)]
pub trait __InstantiateComponent<G: GenericNode>: Component<G> {
    /// Alias to [`instantiate_component`]. For use in proc-macro output.
    ///
    /// The double underscores (`__`) are to prevent conflicts with other trait methods. This is
    /// because we cannot use fully qualified syntax here because it prevents type inference.
    #[doc(hidden)]
    fn __instantiate_component(props: Self::Props) -> View<G>;
}

impl<C, G> __InstantiateComponent<G> for C
where
    C: Component<G>,
    G: GenericNode,
{
    #[inline(always)]
    fn __instantiate_component(props: Self::Props) -> View<G> {
        instantiate_component::<G, C>(props)
    }
}

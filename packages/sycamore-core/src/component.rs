//! Utilities for components and component properties.

use sycamore_reactive::*;

use crate::generic_node::GenericNode;
use crate::view::View;

/// Runs the given closure inside a new component scope. In other words, this does the following:
/// * If hydration is enabled, create a new hydration context.
/// * Create a new untracked scope (see [`untrack`]).
/// * Call the closure `f` passed to this function.
#[doc(hidden)]
pub fn component_scope<G: GenericNode>(f: impl FnOnce() -> View<G>) -> View<G> {
    if G::USE_HYDRATION_CONTEXT {
        #[cfg(feature = "hydrate")]
        return crate::hydrate::hydrate_component(|| untrack(f));
        #[cfg(not(feature = "hydrate"))]
        return untrack(f);
    } else {
        untrack(f)
    }
}

/// A trait that is implemented automatically by the `Prop` derive macro.
pub trait Prop {
    /// The type of the builder. This allows getting the builder type when the name is unknown (e.g.
    /// in a macro).
    type Builder;
    /// Returns the builder for the type.
    /// The builder should be automatically generated using the `Prop` derive macro.
    fn builder() -> Self::Builder;
}

/// Get the builder for the component function.
#[doc(hidden)]
pub fn element_like_component_builder<'a, T: Prop + 'a, G: GenericNode>(
    _f: &impl FnOnce(Scope<'a>, T) -> View<G>,
) -> T::Builder {
    T::builder()
}

/// Component children.
pub struct Children<'a, G: GenericNode> {
    f: Box<dyn FnOnce(BoundedScope<'_, 'a>) -> View<G> + 'a>,
}

impl<'a, F, G: GenericNode> From<F> for Children<'a, G>
where
    F: FnOnce(BoundedScope<'_, 'a>) -> View<G> + 'a,
{
    fn from(f: F) -> Self {
        Self { f: Box::new(f) }
    }
}

impl<'a, G: GenericNode> Children<'a, G> {
    /// Instantiate the child [`View`] with the passed [`Scope`].
    pub fn call(self, cx: BoundedScope<'_, 'a>) -> View<G> {
        (self.f)(cx)
    }

    /// Instantiate the child [`View`] with the passed [`BoundedScope`].
    pub fn call_with_bounded_scope(self, cx: BoundedScope<'_, 'a>) -> View<G> {
        (self.f)(cx)
    }

    /// Create a new [`Children`] from a closure.
    pub fn new(_cx: Scope<'a>, f: impl FnOnce(BoundedScope<'_, 'a>) -> View<G> + 'a) -> Self {
        Self { f: Box::new(f) }
    }
}

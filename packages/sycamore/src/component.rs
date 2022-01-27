//! Utilities for components and component properties.

use crate::generic_node::GenericNode;
use crate::reactive::*;
use crate::view::View;

/// Runs the given closure inside a new component scope. In other words, this does the following:
/// * If hydration is enabled, create a new hydration context.
/// * Create a new untracked scope (see [`untrack`]).
/// * Call the closure `f` passed to this function.
#[doc(hidden)]
pub fn component_scope<G: GenericNode>(f: impl FnOnce() -> View<G>) -> View<G> {
    if G::USE_HYDRATION_CONTEXT {
        #[cfg(feature = "experimental-hydrate")]
        return crate::utils::hydrate::hydrate_component(|| untrack(f));
        #[cfg(not(feature = "experimental-hydrate"))]
        return untrack(f);
    } else {
        untrack(f)
    }
}

/// A trait that is implemented automatically by the [`Prop`](crate::Prop) derive macro.
pub trait Prop {
    type Builder;
    fn builder() -> Self::Builder;
}

/* Implement Prop for () */

/// A builder for `()`.
#[doc(hidden)]
pub struct UnitBuilder;
impl UnitBuilder {
    pub fn build(self) {}
}
impl Prop for () {
    type Builder = UnitBuilder;
    fn builder() -> Self::Builder {
        UnitBuilder
    }
}

/// Get the builder for the component function.
#[doc(hidden)]
pub fn element_like_component_builder<'a, T: Prop + 'a, G: GenericNode>(
    _f: &impl FnOnce(ScopeRef<'a>, T) -> View<G>,
) -> T::Builder {
    T::builder()
}

/// Component children.
pub struct Children<'a, G: GenericNode> {
    f: Box<dyn FnOnce(BoundedScopeRef<'_, 'a>) -> View<G> + 'a>,
}

impl<'a, F, G: GenericNode> From<F> for Children<'a, G>
where
    F: FnOnce(BoundedScopeRef<'_, 'a>) -> View<G> + 'a,
{
    fn from(f: F) -> Self {
        Self { f: Box::new(f) }
    }
}

impl<'a, G: GenericNode> Children<'a, G> {
    pub fn call(self, ctx: ScopeRef<'a>) -> View<G> {
        let mut view = None;
        let _ = ctx.create_child_scope(|ctx| {
            // SAFETY: `self.f` takes the same parameter as ctx.create_child_scope
            let tmp = (self.f)(unsafe { std::mem::transmute(ctx) });
            view = Some(tmp);
        });
        view.unwrap()
    }
}

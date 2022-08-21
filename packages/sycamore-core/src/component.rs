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
///
/// This is used when constructing components in the `view!` macro.
///
/// # Example
/// Deriving an implementation and using the builder to construct an instance of the struct:
/// ```
/// # use sycamore::component::Prop;
/// # use sycamore::prelude::*;
/// #[derive(Prop)]
/// struct ButtonProps {
///     color: String,
///     disabled: bool,
/// }
///
/// let builder = <ButtonProps as Prop>::builder();
/// let button_props = builder.color("red".to_string()).disabled(false).build();
/// ```
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

/// A special property type to allow the component to accept children.
///
/// Add a field called `children` of this type to your properties struct.
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// #[derive(Prop)]
/// struct RowProps<'a, G: Html> {
///     width: i32,
///     children: Children<'a, G>,
/// }
///
/// #[component]
/// fn Row<'a, G: Html>(cx: Scope<'a>, props: RowProps<'a, G>) -> View<G> {
///     // Convert the `Children` into a `View<G>`.
///     let children = props.children.call(cx);
///     view! { cx,
///         div {
///             (children)
///         }
///     }
/// }
///
/// # #[component]
/// # fn App<G: Html>(cx: Scope) -> View<G> {
/// // Using `Row` somewhere else in your app:
/// view! { cx,
///     Row(width=10) {
///         p { "This is a child node." }
///     }
/// }
/// # }
/// ```
pub struct Children<'a, G: GenericNode> {
    f: Box<dyn FnOnce(BoundedScope<'_, 'a>) -> View<G> + 'a>,
}
impl<'a, G: GenericNode> std::fmt::Debug for Children<'a, G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Children").finish()
    }
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

    /// Create a new [`Children`] from a closure.
    pub fn new(_cx: Scope<'a>, f: impl FnOnce(BoundedScope<'_, 'a>) -> View<G> + 'a) -> Self {
        Self { f: Box::new(f) }
    }
}

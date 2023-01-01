//! Utilities for components and component properties.

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
};

use sycamore_reactive::*;

use crate::view::View;
use crate::{generic_node::GenericNode, noderef::NodeRef};

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

/// A trait that is implemented automatically by the `Props` derive macro.
///
/// This is used when constructing components in the `view!` macro.
///
/// # Example
/// Deriving an implementation and using the builder to construct an instance of the struct:
/// ```
/// # use sycamore::component::Props;
/// # use sycamore::prelude::*;
/// #[derive(Props)]
/// struct ButtonProps {
///     color: String,
///     disabled: bool,
/// }
///
/// let builder = <ButtonProps as Props>::builder();
/// let button_props = builder.color("red".to_string()).disabled(false).build();
/// ```
pub trait Props {
    /// The type of the builder. This allows getting the builder type when the name is unknown (e.g.
    /// in a macro).
    type Builder;
    /// Returns the builder for the type.
    /// The builder should be automatically generated using the `Props` derive macro.
    fn builder() -> Self::Builder;
}

/// Make sure that the `Props` trait is implemented for `()` so that components without props can be
/// thought as accepting props of type `()`.
impl Props for () {
    type Builder = UnitBuilder;
    fn builder() -> Self::Builder {
        UnitBuilder
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct UnitBuilder;

impl UnitBuilder {
    pub fn build(self) {}
}

/// A trait that is automatically implemented by all components.
pub trait Component<'a, T: Props, G: GenericNode, S> {
    /// Instantiate the component with the given props and reactive scope.
    fn create(self, cx: Scope<'a>, props: T) -> View<G>;
}
impl<'a, F, T: Props, G: GenericNode> Component<'a, T, G, ((),)> for F
where
    F: FnOnce(Scope<'a>, T) -> View<G>,
{
    fn create(self, cx: Scope<'a>, props: T) -> View<G> {
        self(cx, props)
    }
}
impl<'a, F, G: GenericNode> Component<'a, (), G, ()> for F
where
    F: FnOnce(Scope<'a>) -> View<G>,
{
    fn create(self, cx: Scope<'a>, _props: ()) -> View<G> {
        self(cx)
    }
}

/// Get the builder for the component function.
#[doc(hidden)]
pub fn element_like_component_builder<'a, G: GenericNode, T: Props, S>(
    _f: &impl Component<'a, T, G, S>,
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
/// #[derive(Props)]
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
impl<'a, G: GenericNode> fmt::Debug for Children<'a, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<'a, G: GenericNode> From<View<G>> for Children<'a, G> {
    fn from(view: View<G>) -> Self {
        Self {
            f: Box::new(|_| view),
        }
    }
}

impl<'a, G: GenericNode> Default for Children<'a, G> {
    fn default() -> Self {
        Self {
            f: Box::new(|_| View::default()),
        }
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

pub enum AttributeValue<'cx, G: GenericNode> {
    Str(&'static str),
    DynamicStr(&'cx ReadSignal<String>),
    Bool(bool),
    DynamicBool(&'cx ReadSignal<bool>),
    DangerouslySetInnerHtml(String),
    DynamicDangerouslySetInnerHtml(&'cx ReadSignal<String>),
    Event(&'static str, Box<dyn FnMut(G::EventType)>),
    BindBool(&'static str, &'cx Signal<bool>),
    BindNumber(&'static str, &'cx Signal<f64>),
    BindString(&'static str, &'cx Signal<String>),
    // TODO: Allow Property
    Ref(&'cx NodeRef<G>),
}

impl<'a, G: GenericNode> fmt::Debug for AttributeValue<'a, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttributeValue").finish()
    }
}

#[derive(Default, Debug)]
pub struct Attributes<'cx, G: GenericNode> {
    attrs: HashMap<Cow<'static, str>, AttributeValue<'cx, G>>,
}

impl<'cx, G: GenericNode> Attributes<'cx, G> {
    pub fn new(attributes: HashMap<Cow<'static, str>, AttributeValue<'cx, G>>) -> Self {
        Self { attrs: attributes }
    }
}

impl<'cx, G: GenericNode> Deref for Attributes<'cx, G> {
    type Target = HashMap<Cow<'static, str>, AttributeValue<'cx, G>>;

    fn deref(&self) -> &Self::Target {
        &self.attrs
    }
}

impl<'cx, G: GenericNode> DerefMut for Attributes<'cx, G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.attrs
    }
}

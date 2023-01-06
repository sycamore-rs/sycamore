//! Utilities for components and component properties.

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Display},
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

/// The value of a passthrough attribute.
/// The default for unknown attributes is [`AttributeValue::Str`] or [`AttributeValue::DynamicStr`]
pub enum AttributeValue<'cx, G: GenericNode> {
    /// A string literal value. Example: `attr:id = "test"`
    Str(&'static str),
    /// A dynamic string value from a variable. Example: `attr:id = id_signal`
    DynamicStr(Box<dyn FnMut() -> String + 'cx>),
    /// A boolean literal value. Example: `attr:disabled = true`
    Bool(bool),
    /// A reactive boolean value. Example: `attr:disabled = disabled_signal`
    DynamicBool(&'cx ReadSignal<bool>),
    /// Dangerously set inner HTML with a literal string value.
    DangerouslySetInnerHtml(&'static str),
    /// Dangerously set inner HTML with a dynamic value.
    DynamicDangerouslySetInnerHtml(Box<dyn Display>),
    /// An event binding
    Event(&'static str, Box<dyn FnMut(G::AnyEventData)>),
    /// A binding to a boolean value
    BindBool(&'static str, &'cx Signal<bool>),
    /// A binding to a numeric value
    BindNumber(&'static str, &'cx Signal<f64>),
    /// A binding to a string value
    BindString(&'static str, &'cx Signal<String>),
    /// A property value.
    Property(&'static str, G::PropertyType),
    /// A [`NodeRef`] value.
    Ref(&'cx NodeRef<G>),
}

impl<'a, G: GenericNode> fmt::Debug for AttributeValue<'a, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttributeValue").finish()
    }
}

/// A special property type to allow the component to accept passthrough attributes.
/// This can be useful if your component wraps an HTML element, i.e. accessible component libraries.
///
/// Add a field called `attributes` of this type to your properties struct.
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// #[derive(Props)]
/// struct RowProps<'a, G: Html> {
///     width: i32,
///     children: Children<'a, G>,
///     attributes: Attributes<'a, G>,
/// }
///
/// #[component]
/// fn Row<'a, G: Html>(cx: Scope<'a>, mut props: RowProps<'a, G>) -> View<G> {
///     let children = props.children.call(cx);
///     // Spread the `Attributes` onto the div.
///     view! { cx,
///         div(..props.attributes) {
///             (children)
///         }
///     }
/// }
///
/// # #[component]
/// # fn App<G: Html>(cx: Scope) -> View<G> {
/// // Using `Row` somewhere else in your app:
/// view! { cx,
///     Row(width=10, attr:id = "row1", attr:class = "bg-neutral-400") {
///         p { "This is a child node." }
///     }
/// }
/// # }
/// ```
pub struct Attributes<'cx, G: GenericNode> {
    attrs: HashMap<Cow<'static, str>, AttributeValue<'cx, G>>,
}

impl<'cx, G: GenericNode> Default for Attributes<'cx, G> {
    fn default() -> Self {
        Self {
            attrs: Default::default(),
        }
    }
}

impl<'cx, G: GenericNode> Attributes<'cx, G> {
    // Creates a new [`Attributes`] struct from a map of keys and values.
    pub fn new(attributes: HashMap<Cow<'static, str>, AttributeValue<'cx, G>>) -> Self {
        Self { attrs: attributes }
    }
}

impl<'cx, G: GenericNode> Attributes<'cx, G> {
    /// Read the string value of an attribute. Returns `Option::None` if the attribute is missing
    /// or not a string.
    pub fn get_str(&mut self, key: &str) -> Option<Cow<'static, str>> {
        match self.attrs.get_mut(key)? {
            AttributeValue::Str(s) => Some(Cow::Borrowed(s)),
            AttributeValue::DynamicStr(s) => Some(Cow::Owned(s())),
            _ => None,
        }
    }

    /// Remove an attribute and return the string value of it. Returns `Option::None` if the
    /// attribute is missing or not a string.
    pub fn remove_str(&mut self, key: &str) -> Option<Cow<'static, str>> {
        match self.remove(key)? {
            AttributeValue::Str(s) => Some(Cow::Borrowed(s)),
            AttributeValue::DynamicStr(mut s) => Some(Cow::Owned(s())),
            _ => None,
        }
    }

    /// Read the boolean value of an attribute. Returns `Option::None` if the attribute is missing
    /// or not a boolean.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            AttributeValue::Bool(b) => Some(*b),
            AttributeValue::DynamicBool(b) => Some(*b.get()),
            _ => None,
        }
    }

    /// Remove an attribute and return the boolean value of it. Returns `Option::None` if the
    /// attribute is missing or not a boolean.
    pub fn remove_bool(&mut self, key: &str) -> Option<bool> {
        match self.remove(key)? {
            AttributeValue::Bool(b) => Some(b),
            AttributeValue::DynamicBool(b) => Some(*b.get()),
            _ => None,
        }
    }

    /// Fetch the `dangerously_set_inner_html` attribute from the attributes if it exists.
    pub fn get_dangerously_set_inner_html(&self) -> Option<Cow<'static, str>> {
        match self.get("dangerously_set_inner_html")? {
            AttributeValue::DangerouslySetInnerHtml(html) => Some(Cow::Borrowed(html)),
            AttributeValue::DynamicDangerouslySetInnerHtml(html) => {
                Some(Cow::Owned(html.to_string()))
            }
            _ => None,
        }
    }

    /// Remove the `dangerously_set_inner_html` attribute from the attributes and return its previous value.
    pub fn remove_dangerously_set_inner_html(&mut self) -> Option<Cow<'static, str>> {
        match self.remove("dangerously_set_inner_html")? {
            AttributeValue::DangerouslySetInnerHtml(html) => Some(Cow::Borrowed(html)),
            AttributeValue::DynamicDangerouslySetInnerHtml(html) => {
                Some(Cow::Owned(html.to_string()))
            }
            _ => None,
        }
    }

    /// Fetch the ref from the attributes if it exists.
    pub fn get_ref(&self) -> Option<&'cx NodeRef<G>> {
        match self.get("ref")? {
            AttributeValue::Ref(node_ref) => Some(*node_ref),
            _ => None,
        }
    }

    /// Remove the `ref` from the attributes and return its previous value.
    pub fn remove_ref(&mut self) -> Option<&'cx NodeRef<G>> {
        match self.remove("ref")? {
            AttributeValue::Ref(node_ref) => Some(node_ref),
            _ => None,
        }
    }

    /// Exclude a set of keys from the attributes.
    pub fn exclude_keys(&mut self, keys: &[&str]) {
        for &key in keys {
            self.remove(key);
        }
    }

    /// Get an attribute value if it exists.
    pub fn get(&self, key: &str) -> Option<&AttributeValue<'cx, G>> {
        self.attrs.get(key)
    }

    /// Remove an attribute value and return its previous value
    pub fn remove(&mut self, key: &str) -> Option<AttributeValue<'cx, G>> {
        self.attrs.remove(key)
    }

    /// INTERNAL: used in the `view!` macro to apply attributes
    pub fn drain(
        &mut self,
    ) -> impl Iterator<Item = (Cow<'static, str>, AttributeValue<'cx, G>)> + '_ {
        self.attrs.drain()
    }
}

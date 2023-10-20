//! Utilities for components and component properties.

use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::{self, Display};

use sycamore_reactive::*;

use crate::generic_node::GenericNode;
use crate::noderef::NodeRef;
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
pub trait Component<T: Props, G: GenericNode, S> {
    /// Instantiate the component with the given props and reactive scope.
    fn create(self, props: T) -> View<G>;
}
impl<F, T: Props, G: GenericNode> Component<T, G, ((),)> for F
where
    F: FnOnce(T) -> View<G>,
{
    fn create(self, props: T) -> View<G> {
        self(props)
    }
}
impl<F, G: GenericNode> Component<(), G, ()> for F
where
    F: FnOnce() -> View<G>,
{
    fn create(self, _props: ()) -> View<G> {
        self()
    }
}

/// Get the builder for the component function.
#[doc(hidden)]
pub fn element_like_component_builder<G: GenericNode, T: Props, S>(
    _f: &impl Component<T, G, S>,
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
/// struct RowProps<G: Html> {
///     width: i32,
///     children: Children<G>,
/// }
///
/// #[component]
/// fn Row<G: Html>(props: RowProps<G>) -> View<G> {
///     // Convert the `Children` into a `View<G>`.
///     let children = props.children.call();
///     view! {
///         div {
///             (children)
///         }
///     }
/// }
///
/// # #[component]
/// # fn App<G: Html>() -> View<G> {
/// // Using `Row` somewhere else in your app:
/// view! {
///     Row(width=10) {
///         p { "This is a child node." }
///     }
/// }
/// # }
/// ```
pub struct Children<G: GenericNode> {
    f: Box<dyn FnOnce() -> View<G>>,
}
impl<G: GenericNode> fmt::Debug for Children<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Children").finish()
    }
}

impl<F, G: GenericNode> From<F> for Children<G>
where
    F: FnOnce() -> View<G> + 'static,
{
    fn from(f: F) -> Self {
        Self { f: Box::new(f) }
    }
}

impl<G: GenericNode> From<View<G>> for Children<G> {
    fn from(view: View<G>) -> Self {
        Self {
            f: Box::new(|| view),
        }
    }
}

impl<G: GenericNode> Default for Children<G> {
    fn default() -> Self {
        Self {
            f: Box::new(View::default),
        }
    }
}

impl<G: GenericNode> Children<G> {
    /// Instantiate the child [`View`] with the passed [`Scope`].
    pub fn call(self) -> View<G> {
        (self.f)()
    }

    /// Create a new [`Children`] from a closure.
    pub fn new(f: impl FnOnce() -> View<G> + 'static) -> Self {
        Self { f: Box::new(f) }
    }
}

/// The value of a passthrough attribute.
/// The default for unknown attributes is [`AttributeValue::Str`] or [`AttributeValue::DynamicStr`]
pub enum AttributeValue<G: GenericNode> {
    /// A string literal value. Example: `attr:id = "test"`
    Str(&'static str),
    /// A dynamic string value from a variable. Example: `attr:id = id_signal`
    DynamicStr(Box<dyn FnMut() -> String>),
    /// A boolean literal value. Example: `attr:disabled = true`
    Bool(bool),
    /// A dynamic boolean value from a variable. Example: `attr:disabled = disabled_signal`
    DynamicBool(Box<dyn FnMut() -> bool>),
    /// Dangerously set inner HTML with a literal string value.
    DangerouslySetInnerHtml(&'static str),
    /// Dangerously set inner HTML with a dynamic value.
    DynamicDangerouslySetInnerHtml(Box<dyn Display>),
    /// An event binding
    Event(&'static str, Box<dyn FnMut(G::AnyEventData)>),
    /// A binding to a boolean value
    BindBool(&'static str, Signal<bool>),
    /// A binding to a numeric value
    BindNumber(&'static str, Signal<f64>),
    /// A binding to a string value
    BindString(&'static str, Signal<String>),
    /// A property value.
    Property(&'static str, G::PropertyType),
    /// A [`NodeRef`] value.
    Ref(NodeRef<G>),
}

impl<G: GenericNode> fmt::Debug for AttributeValue<G> {
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
/// struct RowProps<G: Html> {
///     width: i32,
///     children: Children<G>,
///     attributes: Attributes<G>,
/// }
///
/// #[component]
/// fn Row<G: Html>(props: RowProps<G>) -> View<G> {
///     let children = props.children.call();
///     // Spread the `Attributes` onto the div.
///     view! {
///         div(..props.attributes) {
///             (children)
///         }
///     }
/// }
///
/// # #[component]
/// # fn App<G: Html>() -> View<G> {
/// // Using `Row` somewhere else in your app:
/// view! {
///     Row(width=10, attr:id = "row1", attr:class = "bg-neutral-400") {
///         p { "This is a child node." }
///     }
/// }
/// # }
/// ```
#[derive(Debug)]
pub struct Attributes<G: GenericNode> {
    attrs: RefCell<HashMap<Cow<'static, str>, AttributeValue<G>>>,
}

impl<G: GenericNode> Default for Attributes<G> {
    fn default() -> Self {
        Self {
            attrs: RefCell::new(Default::default()),
        }
    }
}

impl<G: GenericNode> Attributes<G> {
    /// Creates a new [`Attributes`] struct from a map of keys and values.
    pub fn new(attributes: HashMap<Cow<'static, str>, AttributeValue<G>>) -> Self {
        Self {
            attrs: RefCell::new(attributes),
        }
    }
}

impl<G: GenericNode> Attributes<G> {
    /// Read the string value of an attribute. Returns `Option::None` if the attribute is missing
    /// or not a string.
    pub fn get_str(&self, key: &str) -> Option<Cow<'static, str>> {
        match self.attrs.borrow_mut().get_mut(key)? {
            AttributeValue::Str(s) => Some(Cow::Borrowed(s)),
            AttributeValue::DynamicStr(s) => Some(Cow::Owned(s())),
            _ => None,
        }
    }

    /// Remove an attribute and return the string value of it. Returns `Option::None` if the
    /// attribute is missing or not a string.
    pub fn remove_str(&self, key: &str) -> Option<Cow<'static, str>> {
        match self.remove(key)? {
            AttributeValue::Str(s) => Some(Cow::Borrowed(s)),
            AttributeValue::DynamicStr(mut s) => Some(Cow::Owned(s())),
            _ => None,
        }
    }

    /// Read the boolean value of an attribute. Returns `Option::None` if the attribute is missing
    /// or not a boolean.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.attrs.borrow_mut().get_mut(key)? {
            AttributeValue::Bool(b) => Some(*b),
            AttributeValue::DynamicBool(b) => Some(b()),
            _ => None,
        }
    }

    /// Remove an attribute and return the boolean value of it. Returns `Option::None` if the
    /// attribute is missing or not a boolean.
    pub fn remove_bool(&self, key: &str) -> Option<bool> {
        match self.remove(key)? {
            AttributeValue::Bool(b) => Some(b),
            AttributeValue::DynamicBool(mut b) => Some(b()),
            _ => None,
        }
    }

    /// Fetch the `dangerously_set_inner_html` attribute from the attributes if it exists.
    pub fn get_dangerously_set_inner_html(&self) -> Option<Cow<'static, str>> {
        match &*self.get("dangerously_set_inner_html")? {
            AttributeValue::DangerouslySetInnerHtml(html) => Some(Cow::Borrowed(html)),
            AttributeValue::DynamicDangerouslySetInnerHtml(html) => {
                Some(Cow::Owned(html.to_string()))
            }
            _ => None,
        }
    }

    /// Remove the `dangerously_set_inner_html` attribute from the attributes and return its
    /// previous value.
    pub fn remove_dangerously_set_inner_html(&self) -> Option<Cow<'static, str>> {
        match self.remove("dangerously_set_inner_html")? {
            AttributeValue::DangerouslySetInnerHtml(html) => Some(Cow::Borrowed(html)),
            AttributeValue::DynamicDangerouslySetInnerHtml(html) => {
                Some(Cow::Owned(html.to_string()))
            }
            _ => None,
        }
    }

    /// Fetch the ref from the attributes if it exists.
    pub fn get_ref(&self) -> Option<Ref<NodeRef<G>>> {
        Ref::filter_map(self.get("ref")?, |value| match value {
            AttributeValue::Ref(node_ref) => Some(node_ref),
            _ => None,
        })
        .ok()
    }

    /// Remove the `ref` from the attributes and return its previous value.
    pub fn remove_ref(&self) -> Option<NodeRef<G>> {
        match self.remove("ref")? {
            AttributeValue::Ref(node_ref) => Some(node_ref),
            _ => None,
        }
    }

    /// Exclude a set of keys from the attributes.
    pub fn exclude_keys(&self, keys: &[&str]) {
        for &key in keys {
            self.remove(key);
        }
    }

    /// Get an attribute value if it exists.
    pub fn get(&self, key: &str) -> Option<Ref<AttributeValue<G>>> {
        Ref::filter_map(self.attrs.borrow(), |attrs| attrs.get(key)).ok()
    }

    /// Remove an attribute value and return its previous value
    pub fn remove(&self, key: &str) -> Option<AttributeValue<G>> {
        self.attrs.borrow_mut().remove(key)
    }

    /// INTERNAL: used in the `view!` macro to apply attributes
    pub fn drain(&self) -> Vec<(Cow<'static, str>, AttributeValue<G>)> {
        self.attrs.borrow_mut().drain().collect()
    }
}

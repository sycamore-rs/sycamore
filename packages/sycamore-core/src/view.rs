//! Abstractions for representing UI views.

use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use sycamore_reactive::*;

use crate::generic_node::GenericNode;

/// Internal type for [`View`]. This is to prevent direct access to the different enum variants.
#[derive(Clone)]
pub(crate) enum ViewType<G: GenericNode> {
    /// A view node.
    Node(G),
    /// A dynamic [`View`].
    Dyn(ReadSignal<View<G>>),
    /// A fragment (aka. list) of [`View`]s.
    Fragment(Rc<[View<G>]>),
}

/// Represents an UI view. Usually constructed using the `view!` macro or using the builder API.
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # #[component]
/// # fn App<G: Html>() -> View<G> {
/// let my_view: View<G> = view! {
///     div {
///         p { "A view." }
///     }
/// };
/// # my_view
/// # }
/// ```
#[derive(Clone)]
pub struct View<G: GenericNode> {
    pub(crate) inner: ViewType<G>,
}

impl<G: GenericNode> View<G> {
    /// Create a new [`View`] from a raw node.
    pub fn new_node(node: G) -> Self {
        Self {
            inner: ViewType::Node(node),
        }
    }

    /// Create a new dynamic [`View`] from a [`FnMut`].
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new_dyn(f: impl FnMut() -> View<G> + 'static) -> Self {
        Self {
            inner: ViewType::Dyn(*create_memo(f).inner_signal()),
        }
    }

    /// Create a new [`View`] fragment from a `Vec` of [`View`]s.
    pub fn new_fragment(fragment: Vec<View<G>>) -> Self {
        Self {
            inner: ViewType::Fragment(Rc::from(fragment)),
        }
    }

    /// Create a new [`View`] with a blank marker node
    ///
    /// Note that this is different from an empty view fragment. Instead, this is a single marker
    /// (dummy) node.
    pub fn empty() -> Self {
        Self::new_node(G::marker())
    }

    /// Try to cast to a [`GenericNode`], or `None` if wrong type.
    pub fn as_node(&self) -> Option<&G> {
        if let ViewType::Node(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    /// Try to cast to a slice representing the view fragment, or `None` if wrong type.
    pub fn as_fragment(&self) -> Option<&[View<G>]> {
        if let ViewType::Fragment(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    /// Try to cast to the underlying [`RcSignal`] for a dynamic view, or `None` if wrong type.
    pub fn as_dyn(&self) -> Option<ReadSignal<View<G>>> {
        if let ViewType::Dyn(v) = &self.inner {
            Some(*v)
        } else {
            None
        }
    }

    /// Returns `true` if the view is a single node. Note that if the view is a fragment containing
    /// only a single child node, this will still return `false`.
    ///
    /// To check whether the [`View`] only contains a single node, use `.flatten().len() == 1`
    /// instead.
    pub fn is_node(&self) -> bool {
        matches!(
            self,
            View {
                inner: ViewType::Node(_)
            }
        )
    }

    /// Returns `true` if the view is a view fragment.
    pub fn is_fragment(&self) -> bool {
        matches!(
            self,
            View {
                inner: ViewType::Fragment(_)
            }
        )
    }

    /// Returns `true` if the view is a dynamic view.
    pub fn is_dyn(&self) -> bool {
        matches!(
            self,
            View {
                inner: ViewType::Dyn(_)
            }
        )
    }

    /// Returns a recursively _flattened_ `Vec` of raw nodes.
    ///
    /// If the current view is dynamic or is a fragment containing dynamic views, the dynamic views
    /// will be accessed reactively.
    pub fn flatten(self) -> Vec<G> {
        match self.inner {
            ViewType::Node(node) => vec![node],
            ViewType::Dyn(lazy) => lazy.get_clone().flatten(),
            ViewType::Fragment(fragment) => {
                fragment.iter().flat_map(|x| x.clone().flatten()).collect()
            }
        }
    }
}

impl<G: GenericNode> Default for View<G> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<G: GenericNode> fmt::Debug for View<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            ViewType::Node(node) => node.fmt(f),
            ViewType::Dyn(lazy) => lazy.with(|x| x.fmt(f)),
            ViewType::Fragment(fragment) => fragment.fmt(f),
        }
    }
}

/// Trait for describing how something should be rendered into DOM nodes.
///
/// A type implementing `ToView` means that it can be converted into a [`View`]. This allows it to
/// be directly interpolated in the `view!` macro.
///
/// # Examples
///
/// Types such as `String`, `&str`, `i32`, and `bool` implement this trait. They will be stringified
/// using the [`ToString`] trait and then converted into a text node.
///
/// ```
/// # use sycamore::prelude::*;
///
/// # fn Component<G: Html>() -> View<G> {
/// let text = "Hello!";
/// view! {
///     (text)
/// }
/// # }
/// ```
///
/// Another type that implements this trait is `Option<View<G>>`. If the value is `Some`, it will be
/// unwrapped. If the value is `None`, an empty view will be created.
///
/// ```
/// # use sycamore::prelude::*;
///
/// # fn Component<G: Html>() -> View<G> {
/// let show = true;
/// view! {
///     (show.then(|| view! { "Hello!" }))
/// }
/// # }
/// ```
pub trait ToView<G: GenericNode> {
    /// Called during the initial render when creating the DOM nodes. Should return a [`View`].
    fn to_view(&self) -> View<G>;
}

impl<G: GenericNode> ToView<G> for View<G> {
    /// Tautology of converting a [`View`] into a [`View`]. This allows us to interpolate views into
    /// other views.
    fn to_view(&self) -> View<G> {
        self.clone()
    }
}

impl<T, G: GenericNode> ToView<G> for Option<T>
where
    T: ToView<G>,
{
    fn to_view(&self) -> View<G> {
        match self {
            Some(v) => v.to_view(),
            None => View::empty(),
        }
    }
}

impl<T, G: GenericNode> ToView<G> for [T]
where
    T: ToView<G>,
{
    fn to_view(&self) -> View<G> {
        View::new_fragment(self.iter().map(ToView::to_view).collect())
    }
}

impl<G: GenericNode> ToView<G> for &'static str {
    fn to_view(&self) -> View<G> {
        View::new_node(G::text_node((*self).into()))
    }
}
impl<G: GenericNode> ToView<G> for Cow<'static, str> {
    fn to_view(&self) -> View<G> {
        View::new_node(G::text_node(self.clone()))
    }
}

macro_rules! impl_to_view_text_to_string {
    ($t:ty) => {
        impl<G: GenericNode> ToView<G> for $t {
            fn to_view(&self) -> View<G> {
                View::new_node(G::text_node(self.to_string().into()))
            }
        }
    };
}

impl_to_view_text_to_string!(String);

impl_to_view_text_to_string!(bool);
impl_to_view_text_to_string!(char);
impl_to_view_text_to_string!(u8);
impl_to_view_text_to_string!(u16);
impl_to_view_text_to_string!(u32);
impl_to_view_text_to_string!(u64);
impl_to_view_text_to_string!(u128);
impl_to_view_text_to_string!(usize);
impl_to_view_text_to_string!(i8);
impl_to_view_text_to_string!(i16);
impl_to_view_text_to_string!(i32);
impl_to_view_text_to_string!(i64);
impl_to_view_text_to_string!(i128);
impl_to_view_text_to_string!(isize);
impl_to_view_text_to_string!(f32);
impl_to_view_text_to_string!(f64);

impl<T, G: GenericNode> ToView<G> for &T
where
    T: ToView<G>,
{
    fn to_view(&self) -> View<G> {
        (*self).to_view()
    }
}
impl<T, G: GenericNode> ToView<G> for Box<T>
where
    T: ToView<G>,
{
    fn to_view(&self) -> View<G> {
        self.as_ref().to_view()
    }
}

impl<T, G: GenericNode> ToView<G> for Rc<T>
where
    T: ToView<G>,
{
    fn to_view(&self) -> View<G> {
        self.as_ref().to_view()
    }
}

impl<T, G: GenericNode> ToView<G> for Arc<T>
where
    T: ToView<G>,
{
    fn to_view(&self) -> View<G> {
        self.as_ref().to_view()
    }
}

#[cfg(test)]
mod tests {
    use sycamore::prelude::*;
    use sycamore::view::ToView;

    #[test]
    fn to_view_tautology() {
        let view = View::new_node(SsrNode::text_node("Hello!".into()));
        assert_eq!(view.to_view().as_node(), view.as_node());
    }

    #[test]
    fn to_view_option() {
        let option = Some("Hello!");
        let view = option.to_view();
        assert!(view.as_node().is_some());
        let string = sycamore::render_to_string(|| view);
        assert_eq!(string, "Hello!");
    }

    #[test]
    fn to_view_fragment() {
        let fragment = vec!["Hello", " ", "World!"];
        let view = fragment.to_view();
        assert!(view.as_fragment().is_some());
        let string = sycamore::render_to_string(|| view);
        assert_eq!(string, "Hello World!");
    }
}

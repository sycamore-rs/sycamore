//! Abstractions for representing UI views.

use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use sycamore_reactive::*;

use crate::generic_node::GenericNode;

/// Internal type for [`View`]. This is to prevent direct access to the different enum variants.
#[derive(Clone)]
pub(crate) enum ViewType<G: GenericNode> {
    /// A view node.
    Node(G),
    /// A dynamic [`View`].
    Dyn(RcSignal<View<G>>),
    /// A fragment (aka. list) of [`View`]s.
    #[allow(clippy::redundant_allocation)] // Cannot create a `Rc<[T]>` directly.
    Fragment(Rc<Box<[View<G>]>>),
}

/// Represents an UI view. Usually constructed using the `view!` macro or using the builder API.
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # #[component]
/// # fn App<G: Html>(cx: Scope) -> View<G> {
/// let my_view: View<G> = view! { cx,
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
    pub fn new_dyn<'a>(cx: Scope<'a>, mut f: impl FnMut() -> View<G> + 'a) -> Self {
        let signal = create_ref(cx, RefCell::new(None::<RcSignal<View<G>>>));
        create_effect(cx, move || {
            let view = f();
            if signal.borrow().is_some() {
                signal.borrow().as_ref().unwrap().set(view);
            } else {
                *signal.borrow_mut() = Some(create_rc_signal(view));
            }
        });
        Self {
            inner: ViewType::Dyn(signal.borrow().as_ref().unwrap().clone()),
        }
    }

    /// Create a new [`View`] from a [`FnMut`] while creating a new child reactive scope.
    pub fn new_dyn_scoped<'a>(
        cx: Scope<'a>,
        mut f: impl FnMut(BoundedScope<'_, 'a>) -> View<G> + 'a,
    ) -> Self {
        let signal = create_ref(cx, RefCell::new(None::<RcSignal<View<G>>>));
        create_effect_scoped(cx, move |cx| {
            let view = f(cx);
            if signal.borrow().is_some() {
                signal.borrow().as_ref().unwrap().set(view);
            } else {
                *signal.borrow_mut() = Some(create_rc_signal(view));
            }
        });
        Self {
            inner: ViewType::Dyn(signal.borrow().as_ref().unwrap().clone()),
        }
    }

    /// Create a new [`View`] fragment from a `Vec` of [`View`]s.
    pub fn new_fragment(fragment: Vec<View<G>>) -> Self {
        Self {
            inner: ViewType::Fragment(Rc::from(fragment.into_boxed_slice())),
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
    pub fn as_dyn(&self) -> Option<&RcSignal<View<G>>> {
        if let ViewType::Dyn(v) = &self.inner {
            Some(v)
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
            ViewType::Dyn(lazy) => lazy.get().as_ref().clone().flatten(),
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
            ViewType::Dyn(lazy) => lazy.get().fmt(f),
            ViewType::Fragment(fragment) => fragment.fmt(f),
        }
    }
}

/// Trait for describing how something should be rendered into DOM nodes.
///
/// A type implementing `IntoView` means that it can be converted into a [`View`]. This allows it to
/// be directly interpolated in the `view!` macro.
pub trait IntoView<G: GenericNode> {
    /// Called during the initial render when creating the DOM nodes. Should return a [`View`].
    fn create(&self) -> View<G>;
}

impl<G: GenericNode> IntoView<G> for View<G> {
    /// Tautology of converting a [`View`] into a [`View`]. This allows us to interpolate views into
    /// other views.
    fn create(&self) -> View<G> {
        self.clone()
    }
}
impl<G: GenericNode> IntoView<G> for &View<G> {
    fn create(&self) -> View<G> {
        (*self).clone()
    }
}

impl<T: fmt::Display + 'static, G: GenericNode> IntoView<G> for T {
    fn create(&self) -> View<G> {
        // Workaround for specialization.
        // Inspecting the type is optimized away at compile time.

        macro_rules! specialize_as_ref_to_str {
            ($($t: ty),*) => {
                $(
                    if let Some(s) = <dyn Any>::downcast_ref::<$t>(self) {
                        return View::new_node(G::text_node(s.as_ref()));
                    }
                )*
            }
        }

        macro_rules! specialize_num {
            ($($t: ty),*) => {
                $(
                    if let Some(&n) = <dyn Any>::downcast_ref::<$t>(self) {
                        return View::new_node(G::text_node_int(n as i32));
                    }
                )*
            }
        }

        macro_rules! specialize_big_num {
            ($($t: ty),*) => {
                $(
                    if let Some(&n) = <dyn Any>::downcast_ref::<$t>(self) {
                        if n <= i32::MAX as $t {
                            return View::new_node(G::text_node_int(n as i32));
                        } else {
                            return View::new_node(G::text_node(&n.to_string()));
                        }
                    }
                )*
            }
        }

        // Strings and string slices.
        specialize_as_ref_to_str!(&str, String, Rc<str>, Rc<String>, Cow<'_, str>);

        // Numbers that are smaller than can be represented by an `i32` use fast-path by passing
        // value directly to JS. Note that `u16` and `u32` cannot be represented by an `i32`
        specialize_num!(i8, i16, i32, u8);
        // Number that are bigger than an `i32`.
        specialize_big_num!(i64, i128, isize, u16, u32, u64, u128, usize);

        // Generic slow-path.
        let t = self.to_string();
        View::new_node(G::text_node(&t))
    }
}

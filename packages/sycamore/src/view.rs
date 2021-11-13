//! Result of the [view!](crate::view!) macro.

use std::any::Any;
use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;

use crate::generic_node::GenericNode;
use crate::reactive::{create_memo, ReadSignal};

/// Internal type for [`View`].
#[derive(Clone)]
pub(crate) enum ViewType<G: GenericNode> {
    /// A DOM node.
    Node(G),
    /// A dynamic [`View`].
    Dyn(ReadSignal<View<G>>),
    /// A fragment of [`View`]s.
    #[allow(clippy::redundant_allocation)] // Cannot create a `Rc<[T]>` directly.
    Fragment(Rc<Box<[View<G>]>>),
}

/// Result of the [view!](crate::view!) macro.
#[derive(Clone)]
pub struct View<G: GenericNode> {
    pub(crate) inner: ViewType<G>,
}

impl<G: GenericNode> View<G> {
    /// Create a new [`View`] from a [`GenericNode`].
    pub fn new_node(node: G) -> Self {
        Self {
            inner: ViewType::Node(node),
        }
    }

    /// Create a new [`View`] from a [`FnMut`].
    pub fn new_dyn(f: impl FnMut() -> View<G> + 'static) -> Self {
        let memo = create_memo(f);
        Self {
            inner: ViewType::Dyn(memo),
        }
    }

    /// Create a new [`View`] from a `Vec` of [`GenericNode`]s.
    pub fn new_fragment(fragment: Vec<View<G>>) -> Self {
        Self {
            inner: ViewType::Fragment(Rc::from(fragment.into_boxed_slice())),
        }
    }

    /// Create a new [`View`] with a blank comment node
    pub fn empty() -> Self {
        Self::new_node(G::marker())
    }

    pub fn as_node(&self) -> Option<&G> {
        if let ViewType::Node(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_fragment(&self) -> Option<&[View<G>]> {
        if let ViewType::Fragment(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn as_dyn(&self) -> Option<&ReadSignal<View<G>>> {
        if let ViewType::Dyn(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    pub fn is_node(&self) -> bool {
        matches!(
            self,
            View {
                inner: ViewType::Node(_)
            }
        )
    }

    pub fn is_fragment(&self) -> bool {
        matches!(
            self,
            View {
                inner: ViewType::Fragment(_)
            }
        )
    }

    pub fn is_dyn(&self) -> bool {
        matches!(
            self,
            View {
                inner: ViewType::Dyn(_)
            }
        )
    }

    /// Returns a `Vec` of nodes.
    pub fn flatten(self) -> Vec<G> {
        match self.inner {
            ViewType::Node(node) => vec![node],
            ViewType::Dyn(lazy) => lazy.get().as_ref().clone().flatten(),
            ViewType::Fragment(fragment) => fragment
                .iter()
                .map(|x| x.clone().flatten())
                .flatten()
                .collect(),
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
pub trait IntoView<G: GenericNode> {
    /// Called during the initial render when creating the DOM nodes. Should return a
    /// `Vec` of [`GenericNode`]s.
    fn create(&self) -> View<G>;
}

impl<G: GenericNode> IntoView<G> for View<G> {
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
            ($t: ty) => {{
                if let Some(s) = <dyn Any>::downcast_ref::<$t>(self) {
                    return View::new_node(G::text_node(s.as_ref()));
                }
            }};
            ($t: ty, $($rest: ty),*) => {{
                specialize_as_ref_to_str!($t);
                specialize_as_ref_to_str!($($rest),*);
            }};
        }

        macro_rules! specialize_num_with_lexical {
            ($t: ty) => {{
                if let Some(&n) = <dyn Any>::downcast_ref::<$t>(self) {
                    return View::new_node(G::text_node(&lexical::to_string(n)));
                }
            }};
            ($t: ty, $($rest: ty),*) => {{
                specialize_num_with_lexical!($t);
                specialize_num_with_lexical!($($rest),*);
            }};
        }

        // Strings and string slices.
        specialize_as_ref_to_str!(&str, String, Rc<str>, Rc<String>, Cow<'_, str>);

        // Numbers use lexical.
        specialize_num_with_lexical!(
            i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
        );

        // Generic slow-path.
        let t = self.to_string();
        View::new_node(G::text_node(&t))
    }
}

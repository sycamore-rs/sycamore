//! Result of the [`template`](crate::template!) macro.

use std::any::Any;
use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;

use crate::generic_node::GenericNode;
use crate::reactive::{create_memo, StateHandle};

/// Internal type for [`Template`].
#[derive(Clone)]
pub(crate) enum TemplateType<G: GenericNode> {
    /// A DOM node.
    Node(G),
    /// A dynamic [`Template`].
    Dyn(StateHandle<Template<G>>),
    /// A fragment of [`Template`]s.
    #[allow(clippy::redundant_allocation)] // Cannot create a `Rc<[T]>` directly.
    Fragment(Rc<Box<[Template<G>]>>),
}

/// Result of the [`template`](crate::template!) macro. Should not be constructed manually.
#[derive(Clone)]
pub struct Template<G: GenericNode> {
    pub(crate) inner: TemplateType<G>,
}

impl<G: GenericNode> Template<G> {
    /// Create a new [`Template`] from a [`GenericNode`].
    pub fn new_node(node: G) -> Self {
        Self {
            inner: TemplateType::Node(node),
        }
    }

    /// Create a new [`Template`] from a [`FnMut`].
    pub fn new_dyn(f: impl FnMut() -> Template<G> + 'static) -> Self {
        let memo = create_memo(f);
        Self {
            inner: TemplateType::Dyn(memo),
        }
    }

    /// Create a new [`Template`] from a `Vec` of [`GenericNode`]s.
    pub fn new_fragment(fragment: Vec<Template<G>>) -> Self {
        Self {
            inner: TemplateType::Fragment(Rc::from(fragment.into_boxed_slice())),
        }
    }

    /// Create a new [`Template`] with a blank comment node
    pub fn empty() -> Self {
        Self::new_node(G::marker())
    }

    pub fn as_node(&self) -> Option<&G> {
        if let TemplateType::Node(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_fragment(&self) -> Option<&[Template<G>]> {
        if let TemplateType::Fragment(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn as_dyn(&self) -> Option<&StateHandle<Template<G>>> {
        if let TemplateType::Dyn(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    pub fn is_node(&self) -> bool {
        matches!(
            self,
            Template {
                inner: TemplateType::Node(_)
            }
        )
    }

    pub fn is_fragment(&self) -> bool {
        matches!(
            self,
            Template {
                inner: TemplateType::Fragment(_)
            }
        )
    }

    pub fn is_dyn(&self) -> bool {
        matches!(
            self,
            Template {
                inner: TemplateType::Dyn(_)
            }
        )
    }

    /// Returns a `Vec` of nodes.
    pub fn flatten(self) -> Vec<G> {
        match self.inner {
            TemplateType::Node(node) => vec![node],
            TemplateType::Dyn(lazy) => lazy.get().as_ref().clone().flatten(),
            TemplateType::Fragment(fragment) => fragment
                .iter()
                .map(|x| x.clone().flatten())
                .flatten()
                .collect(),
        }
    }
}

impl<G: GenericNode> Default for Template<G> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<G: GenericNode> fmt::Debug for Template<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            TemplateType::Node(node) => node.fmt(f),
            TemplateType::Dyn(lazy) => lazy.get().fmt(f),
            TemplateType::Fragment(fragment) => fragment.fmt(f),
        }
    }
}

/// Trait for describing how something should be rendered into DOM nodes.
pub trait IntoTemplate<G: GenericNode> {
    /// Called during the initial render when creating the DOM nodes. Should return a
    /// `Vec` of [`GenericNode`]s.
    fn create(&self) -> Template<G>;
}

impl<G: GenericNode> IntoTemplate<G> for Template<G> {
    fn create(&self) -> Template<G> {
        self.clone()
    }
}

impl<G: GenericNode> IntoTemplate<G> for &Template<G> {
    fn create(&self) -> Template<G> {
        (*self).clone()
    }
}

impl<T: fmt::Display + 'static, G: GenericNode> IntoTemplate<G> for T {
    fn create(&self) -> Template<G> {
        // Workaround for specialization.
        // Inspecting the type is optimized away at compile time.

        macro_rules! specialize_as_ref_to_str {
            ($t: ty) => {{
                if let Some(s) = <dyn Any>::downcast_ref::<$t>(self) {
                    return Template::new_node(G::text_node(s.as_ref()));
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
                    return Template::new_node(G::text_node(&lexical::to_string(n)));
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
        Template::new_node(G::text_node(&t))
    }
}

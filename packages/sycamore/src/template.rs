//! Result of the [`template`](crate::template!) macro.

use std::any::Any;
use std::fmt;
use std::rc::Rc;

use crate::generic_node::GenericNode;
use crate::reactive::effect::create_memo;
use crate::reactive::signal::ReadSignal;

/// Internal type for [`Template`].
#[derive(Clone)]
pub(crate) enum TemplateType<G: GenericNode> {
    /// A DOM node.
    Node(G),
    /// A dynamic [`Template`].
    Dyn(ReadSignal<Template<G>>),
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
    pub fn as_dyn(&self) -> Option<&ReadSignal<Template<G>>> {
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

impl<T: fmt::Display + 'static, G: GenericNode> IntoTemplate<G> for T {
    fn create(&self) -> Template<G> {
        if let Some(str) = <dyn Any>::downcast_ref::<&str>(self) {
            Template::new_node(G::text_node(str))
        } else {
            Template::new_node(G::text_node(&self.to_string()))
        }
    }
}

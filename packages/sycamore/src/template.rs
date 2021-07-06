//! Result of the [`template`](crate::template!) macro.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::generic_node::GenericNode;

/// Internal type for [`Template`].
#[derive(Clone)]
pub(crate) enum TemplateType<G: GenericNode> {
    /// A DOM node.
    Node(G),
    /// A lazy-computed [`Template`].
    Lazy(Rc<RefCell<dyn FnMut() -> Template<G>>>),
    /// A fragment of [`Template`]s.
    Fragment(Vec<Template<G>>),
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
    pub fn new_lazy(f: impl FnMut() -> Template<G> + 'static) -> Self {
        Self {
            inner: TemplateType::Lazy(Rc::new(RefCell::new(f))),
        }
    }

    /// Create a new [`Template`] from a `Vec` of [`GenericNode`]s.
    pub fn new_fragment(fragment: Vec<Template<G>>) -> Self {
        Self {
            inner: TemplateType::Fragment(fragment),
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

    pub fn as_fragment(&self) -> Option<&Vec<Template<G>>> {
        if let TemplateType::Fragment(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn as_lazy(&self) -> Option<&Rc<RefCell<dyn FnMut() -> Template<G>>>> {
        if let TemplateType::Lazy(v) = &self.inner {
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

    pub fn is_lazy(&self) -> bool {
        matches!(
            self,
            Template {
                inner: TemplateType::Lazy(_)
            }
        )
    }

    pub fn append_template(&mut self, template: Template<G>) {
        match &mut self.inner {
            TemplateType::Node(node) => {
                self.inner =
                    TemplateType::Fragment(vec![Template::new_node(node.clone()), template]);
            }
            TemplateType::Lazy(lazy) => {
                self.inner = TemplateType::Fragment(vec![
                    Template {
                        inner: TemplateType::Lazy(Rc::clone(&lazy)),
                    },
                    template,
                ]);
            }
            TemplateType::Fragment(fragment) => {
                fragment.push(template);
            }
        }
    }

    /// Returns a `Vec` of nodes. Lazy nodes are evaluated.
    // #[deprecated(note = "footgun when rendering")]
    // TODO: re-enable
    pub fn flatten(self) -> Vec<G> {
        match self.inner {
            TemplateType::Node(node) => vec![node],
            TemplateType::Lazy(lazy) => lazy.borrow_mut()().flatten(),
            TemplateType::Fragment(fragment) => fragment
                .into_iter()
                .map(|x| x.flatten())
                .flatten()
                .collect(),
        }
    }
}

impl<G: GenericNode> fmt::Debug for Template<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            TemplateType::Node(node) => node.fmt(f),
            TemplateType::Lazy(lazy) => lazy.as_ref().borrow_mut()().fmt(f),
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

impl<T: fmt::Display + ?Sized, G: GenericNode> IntoTemplate<G> for T {
    fn create(&self) -> Template<G> {
        Template::new_node(G::text_node(&format!("{}", self)))
    }
}

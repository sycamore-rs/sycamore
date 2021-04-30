//! Result of the [`template`](crate::template) macro.

use std::cell::RefCell;
use std::rc::Rc;
use std::{fmt, mem};

use crate::generic_node::GenericNode;

/// Internal type for [`TemplateResult`].
#[derive(Clone)]
pub(crate) enum TemplateResultInner<G: GenericNode> {
    /// A DOM node.
    Node(G),
    /// A lazy-computed [`TemplateResult`].
    Lazy(Option<Rc<RefCell<dyn FnMut() -> TemplateResult<G>>>>),
    /// A fragment of [`TemplateResult`]s.
    Fragment(Vec<TemplateResult<G>>),
}

/// Result of the [`template`](crate::template) macro. Should not be constructed manually.
#[derive(Clone)]
pub struct TemplateResult<G: GenericNode> {
    pub(crate) inner: TemplateResultInner<G>,
}

impl<G: GenericNode> TemplateResult<G> {
    /// Create a new [`TemplateResult`] from a [`GenericNode`].
    pub fn new_node(node: G) -> Self {
        Self {
            inner: TemplateResultInner::Node(node),
        }
    }

    /// Create a new [`TemplateResult`] from a [`FnOnce`].
    pub fn new_lazy(f: impl FnMut() -> TemplateResult<G> + 'static) -> Self {
        Self {
            inner: TemplateResultInner::Lazy(Some(Rc::new(RefCell::new(f)))),
        }
    }

    /// Create a new [`TemplateResult`] from a `Vec` of [`GenericNode`]s.
    pub fn new_fragment(fragment: Vec<TemplateResult<G>>) -> Self {
        Self {
            inner: TemplateResultInner::Fragment(fragment),
        }
    }

    /// Create a new [`TemplateResult`] with a blank comment node
    pub fn empty() -> Self {
        Self::new_node(G::marker())
    }

    pub fn append_template(&mut self, template: TemplateResult<G>) {
        match &mut self.inner {
            TemplateResultInner::Node(node) => {
                self.inner = TemplateResultInner::Fragment(vec![
                    TemplateResult::new_node(node.clone()),
                    template,
                ])
            }
            TemplateResultInner::Lazy(lazy) => {
                self.inner = TemplateResultInner::Fragment(vec![
                    TemplateResult {
                        inner: TemplateResultInner::Lazy(mem::take(lazy)),
                    },
                    template,
                ])
            }
            TemplateResultInner::Fragment(fragment) => {
                fragment.push(template);
            }
        }
    }

    /// Returns a `Vec` of nodes. Lazy nodes are evaluated.
    pub fn flatten(self) -> Vec<G> {
        match self.inner {
            TemplateResultInner::Node(node) => vec![node],
            TemplateResultInner::Lazy(lazy) => lazy.unwrap().borrow_mut()().flatten(),
            TemplateResultInner::Fragment(fragment) => fragment
                .into_iter()
                .map(|x| x.flatten())
                .flatten()
                .collect(),
        }
    }
}

impl<G: GenericNode> fmt::Debug for TemplateResult<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            TemplateResultInner::Node(node) => node.fmt(f),
            TemplateResultInner::Lazy(lazy) => lazy.as_ref().unwrap().borrow_mut()().fmt(f),
            TemplateResultInner::Fragment(fragment) => fragment.fmt(f),
        }
    }
}

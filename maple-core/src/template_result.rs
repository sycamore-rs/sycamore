//! Result of the [`template`](crate::template) macro.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::generic_node::GenericNode;

/// Internal type for [`TemplateResult`].
#[derive(Clone)]
pub(crate) enum TemplateResultInner<G: GenericNode> {
    /// A DOM node.
    Node(G),
    /// A lazy-computed [`TemplateResult`].
    Lazy(Rc<RefCell<dyn FnMut() -> TemplateResult<G>>>),
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
            inner: TemplateResultInner::Lazy(Rc::new(RefCell::new(f))),
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

    pub fn as_node(&self) -> Option<&G> {
        if let TemplateResultInner::Node(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_fragment(&self) -> Option<&Vec<TemplateResult<G>>> {
        if let TemplateResultInner::Fragment(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn as_lazy(&self) -> Option<&Rc<RefCell<dyn FnMut() -> TemplateResult<G>>>> {
        if let TemplateResultInner::Lazy(v) = &self.inner {
            Some(v)
        } else {
            None
        }
    }

    pub fn is_node(&self) -> bool {
        matches!(
            self,
            TemplateResult {
                inner: TemplateResultInner::Node(_)
            }
        )
    }

    pub fn is_fragment(&self) -> bool {
        matches!(
            self,
            TemplateResult {
                inner: TemplateResultInner::Fragment(_)
            }
        )
    }

    pub fn is_lazy(&self) -> bool {
        matches!(
            self,
            TemplateResult {
                inner: TemplateResultInner::Lazy(_)
            }
        )
    }

    pub fn append_template(&mut self, template: TemplateResult<G>) {
        match &mut self.inner {
            TemplateResultInner::Node(node) => {
                self.inner = TemplateResultInner::Fragment(vec![
                    TemplateResult::new_node(node.clone()),
                    template,
                ]);
            }
            TemplateResultInner::Lazy(lazy) => {
                self.inner = TemplateResultInner::Fragment(vec![
                    TemplateResult {
                        inner: TemplateResultInner::Lazy(Rc::clone(&lazy)),
                    },
                    template,
                ]);
            }
            TemplateResultInner::Fragment(fragment) => {
                fragment.push(template);
            }
        }
    }

    /// Returns a `Vec` of nodes. Lazy nodes are evaluated.
    // #[deprecated(note = "footgun when rendering")]
    // TODO: re-enable
    pub fn flatten(self) -> Vec<G> {
        match self.inner {
            TemplateResultInner::Node(node) => vec![node],
            TemplateResultInner::Lazy(lazy) => lazy.borrow_mut()().flatten(),
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
            TemplateResultInner::Lazy(lazy) => lazy.as_ref().borrow_mut()().fmt(f),
            TemplateResultInner::Fragment(fragment) => fragment.fmt(f),
        }
    }
}

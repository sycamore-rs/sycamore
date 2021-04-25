//! Result of the [`template`](crate::template) macro.

use std::mem;
use std::rc::Rc;

use crate::generic_node::GenericNode;

/// Internal type for [`TemplateResult`].
#[derive(Clone)]
pub enum TemplateResultInner<G: GenericNode> {
    /// A DOM node.
    Node(G),
    /// A lazy-computed [`TemplateResult`].
    Lazy(Option<Rc<dyn FnOnce() -> TemplateResult<G>>>),
    /// A fragment of [`TemplateResult`]s.
    Fragment(Vec<TemplateResult<G>>),
}

/// Result of the [`template`](crate::template) macro. Should not be constructed manually.
#[derive(Clone)]
pub struct TemplateResult<G: GenericNode> {
    inner: TemplateResultInner<G>,
}

impl<G: GenericNode> TemplateResult<G> {
    /// Create a new [`TemplateResult`] from a [`GenericNode`].
    pub fn new_node(node: G) -> Self {
        Self {
            inner: TemplateResultInner::Node(node),
        }
    }

    /// Create a new [`TemplateResult`] from a [`FnOnce`].
    pub fn new_lazy(f: impl FnOnce() -> TemplateResult<G> + 'static) -> Self {
        Self {
            inner: TemplateResultInner::Lazy(Some(Rc::new(f))),
        }
    }

    /// Create a new [`TemplateResult`] from a `Vec` of [`GenericNode`]s.
    pub fn new_fragment(fragment: Vec<TemplateResult<G>>) -> Self {
        debug_assert!(
            !fragment.is_empty(),
            "fragment must have at least 1 child node, use empty() instead"
        );

        Self {
            inner: TemplateResultInner::Fragment(fragment),
        }
    }

    /// Create a new [`TemplateResult`] with a blank comment node
    pub fn empty() -> Self {
        Self::new_node(G::marker())
    }

    /// # Panics
    /// This method panics if variant is a `Lazy` and already executed.
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
}

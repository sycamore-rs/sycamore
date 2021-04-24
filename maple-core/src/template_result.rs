//! Result of the [`template`](crate::template) macro.

use crate::generic_node::GenericNode;

/// Internal type for [`TemplateResult`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateResultInner<G: GenericNode> {
    Node(G),
    Fragment(Vec<TemplateResult<G>>),
}

/// Result of the [`template`](crate::template) macro. Should not be constructed manually.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Flattens the [`TemplateResult`] into a flat `Vec` of nodes.
    pub fn flatten(&self) -> Vec<G> {
        match &self.inner {
            TemplateResultInner::Node(node) => vec![node.clone()],
            TemplateResultInner::Fragment(fragment) => {
                fragment.iter().map(|t| t.flatten()).flatten().collect()
            }
        }
    }

    pub fn append_template(&mut self, template: TemplateResult<G>) {
        match &mut self.inner {
            TemplateResultInner::Node(node) => {
                self.inner = TemplateResultInner::Fragment(vec![
                    TemplateResult::new_node(node.clone()),
                    template,
                ])
            }
            TemplateResultInner::Fragment(fragment) => {
                fragment.push(template);
            }
        }
    }
}

impl<G: GenericNode> IntoIterator for TemplateResult<G> {
    type Item = G;

    type IntoIter = std::vec::IntoIter<G>;

    fn into_iter(self) -> Self::IntoIter {
        self.flatten().into_iter()
    }
}

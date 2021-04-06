use crate::generic_node::GenericNode;

/// Internal type for [`TemplateResult`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateResultType<G: GenericNode> {
    Node(G),
    Fragment(Vec<G>),
}

/// Result of the [`template`] macro. Should not be constructed manually.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateResult<G: GenericNode> {
    node: G,
}

impl<G: GenericNode> TemplateResult<G> {
    /// Create a new [`TemplateResult`] from a [`GenericNode`].
    pub fn new(node: G) -> Self {
        Self { node }
    }

    /// Create a new [`TemplateResult`] with a blank comment node
    pub fn empty() -> Self {
        Self::new(G::marker())
    }

    pub fn inner_node(&self) -> &G {
        &self.node
    }
}

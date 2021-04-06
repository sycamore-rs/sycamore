use crate::generic_node::GenericNode;

/// Result of the [`template`] macro.
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

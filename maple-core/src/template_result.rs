use crate::generic_node::GenericNode;

/// Internal type for [`TemplateResult`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateResultInner<G: GenericNode> {
    Node(G),
    Fragment(Vec<G>),
}

/// Result of the [`template`] macro. Should not be constructed manually.
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
    pub fn new_fragment(fragment: Vec<G>) -> Self {
        Self {
            inner: TemplateResultInner::Fragment(fragment),
        }
    }

    /// Create a new [`TemplateResult`] with a blank comment node
    pub fn empty() -> Self {
        Self::new_node(G::marker())
    }

    #[deprecated]
    pub fn inner_node(&self) -> &G {
        match &self.inner {
            TemplateResultInner::Node(node) => node,
            TemplateResultInner::Fragment(fragment) => fragment.last().unwrap(),
        }
    }

    pub fn first_node(&self) -> &G {
        match &self.inner {
            TemplateResultInner::Node(node) => node,
            TemplateResultInner::Fragment(fragment) => fragment.first().unwrap(),
        }
    }

    pub fn last_node(&self) -> &G {
        match &self.inner {
            TemplateResultInner::Node(node) => node,
            TemplateResultInner::Fragment(fragment) => fragment.last().unwrap(),
        }
    }
}

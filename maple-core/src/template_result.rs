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

    /// Gets the first node in the [`TemplateResult`].
    ///
    /// # Panics
    ///
    /// Panics if the fragment has no child nodes.
    pub fn first_node(&self) -> &G {
        match &self.inner {
            TemplateResultInner::Node(node) => node,
            TemplateResultInner::Fragment(fragment) => {
                fragment.first().expect("fragment has no child nodes")
            }
        }
    }

    /// Gets the last node in the [`TemplateResult`].
    ///
    /// # Panics
    ///
    /// Panics if the fragment has no child nodes.
    pub fn last_node(&self) -> &G {
        match &self.inner {
            TemplateResultInner::Node(node) => node,
            TemplateResultInner::Fragment(fragment) => {
                fragment.last().expect("fragment has no child nodes")
            }
        }
    }

    pub fn iter(&self) -> Iter<G> {
        match &self.inner {
            TemplateResultInner::Node(node) => Iter::Node(Some(node).into_iter()),
            TemplateResultInner::Fragment(fragment) => Iter::Fragment(fragment.iter()),
        }
    }
}

impl<G: GenericNode> IntoIterator for TemplateResult<G> {
    type Item = G;

    type IntoIter = std::vec::IntoIter<G>;

    fn into_iter(self) -> Self::IntoIter {
        match self.inner {
            TemplateResultInner::Node(node) => vec![node].into_iter(),
            TemplateResultInner::Fragment(fragment) => fragment.into_iter(),
        }
    }
}

/// An iterator over references of the nodes in [`TemplateResult`]. Created using [`TemplateResult::iter`].
pub enum Iter<'a, G: GenericNode> {
    Node(std::option::IntoIter<&'a G>),
    Fragment(std::slice::Iter<'a, G>),
}

impl<'a, G: GenericNode> Iterator for Iter<'a, G> {
    type Item = &'a G;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Node(node) => node.next(),
            Iter::Fragment(fragment) => fragment.next(),
        }
    }
}

impl<'a, G: GenericNode> IntoIterator for &'a TemplateResult<G> {
    type Item = &'a G;

    type IntoIter = Iter<'a, G>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

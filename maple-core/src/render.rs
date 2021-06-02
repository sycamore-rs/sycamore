//! Trait for describing how something should be rendered into DOM nodes.

use std::fmt;

use crate::generic_node::GenericNode;
use crate::template_result::TemplateResult;

/// Trait for describing how something should be rendered into DOM nodes.
pub trait IntoTemplate<G: GenericNode> {
    /// Called during the initial render when creating the DOM nodes. Should return a
    /// `Vec` of [`GenericNode`]s.
    fn create(&self) -> TemplateResult<G>;
}

impl<G: GenericNode> IntoTemplate<G> for TemplateResult<G> {
    fn create(&self) -> TemplateResult<G> {
        self.clone()
    }
}

impl<T: fmt::Display + ?Sized, G: GenericNode> IntoTemplate<G> for T {
    fn create(&self) -> TemplateResult<G> {
        TemplateResult::new_node(G::text_node(&format!("{}", self)))
    }
}

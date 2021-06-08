//! Trait for describing how something should be rendered into DOM nodes.

use std::fmt;

use crate::generic_node::GenericNode;
use crate::template::Template;

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

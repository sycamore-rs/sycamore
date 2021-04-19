//! Trait for describing how something should be rendered into DOM nodes.

use std::fmt;

use crate::generic_node::GenericNode;
use crate::template_result::TemplateResult;

/// Trait for describing how something should be rendered into DOM nodes.
pub trait Render<G: GenericNode> {
    /// Called during the initial render when creating the DOM nodes. Should return a
    /// `Vec` of [`GenericNode`]s.
    fn create(&self) -> Vec<G>;

    /// Called when the node should be updated with new state.
    /// The default implementation of this will replace the child node completely with the result of
    /// calling `render` again. Another implementation might be better suited to some specific
    /// types. For example, text nodes can simply replace the inner text instead of recreating a
    /// new node.
    ///
    /// Returns the new node. If the node is reused instead of replaced, the returned node is simply
    /// the node passed in.
    fn update_node<'a>(&self, parent: &G, node: &'a [G]) -> Vec<G> {
        let new_nodes = self.create();

        for new_node in &new_nodes {
            parent.replace_child(new_node, node.first().unwrap());
        }

        new_nodes
    }
}

impl<T: fmt::Display + ?Sized, G: GenericNode> Render<G> for T {
    fn create(&self) -> Vec<G> {
        vec![G::text_node(&format!("{}", self))]
    }

    fn update_node<'a>(&self, _parent: &G, node: &'a [G]) -> Vec<G> {
        // replace `textContent` of `node` instead of recreating

        node.first()
            .unwrap()
            .update_inner_text(&format!("{}", self));

        node.to_vec()
    }
}

impl<G: GenericNode> Render<G> for TemplateResult<G> {
    fn create(&self) -> Vec<G> {
        self.into_iter().cloned().collect()
    }
}

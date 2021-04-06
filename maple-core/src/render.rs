//! Trait for describing how something should be rendered into DOM nodes.

use std::fmt;

use crate::generic_node::GenericNode;
use crate::template_result::TemplateResult;

/// Trait for describing how something should be rendered into DOM nodes.
pub trait Render<G: GenericNode> {
    /// Called during the initial render when creating the DOM nodes. Should return a
    /// [`GenericNode`].
    fn render(&self) -> G;

    /// Called when the node should be updated with new state.
    /// The default implementation of this will replace the child node completely with the result of
    /// calling `render` again. Another implementation might be better suited to some specific
    /// types. For example, text nodes can simply replace the inner text instead of recreating a
    /// new node.
    ///
    /// Returns the new node. If the node is reused instead of replaced, the returned node is simply
    /// the node passed in.
    fn update_node(&self, parent: &G, node: &G) -> G {
        let new_node = self.render();
        parent.replace_child(&new_node, &node);
        new_node
    }
}

impl<T: fmt::Display + ?Sized, G: GenericNode> Render<G> for T {
    fn render(&self) -> G {
        G::text_node(&format!("{}", self))
    }

    fn update_node(&self, _parent: &G, node: &G) -> G {
        // replace `textContent` of `node` instead of recreating

        node.update_inner_text(&format!("{}", self));

        node.clone()
    }
}

impl<G: GenericNode> Render<G> for TemplateResult<G> {
    fn render(&self) -> G {
        self.inner_node().clone()
    }
}

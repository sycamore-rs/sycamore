//! Trait for describing how something should be rendered into DOM nodes.

use std::fmt;
use std::rc::Rc;

use crate::generic_node::GenericNode;
use crate::reactive::VecDiff;
use crate::{TemplateList, TemplateResult};

/// Trait for describing how something should be rendered into DOM nodes.
pub trait Render<G: GenericNode> {
    /// Called during the initial render when creating the DOM nodes. Should return a [`GenericNode`].
    fn render(&self) -> G;

    /// Called when the node should be updated with new state.
    /// The default implementation of this will replace the child node completely with the result of calling `render` again.
    /// Another implementation might be better suited to some specific types.
    /// For example, text nodes can simply replace the inner text instead of recreating a new node.
    ///
    /// Returns the new node. If the node is reused instead of replaced, the returned node is simply the node passed in.
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

        node.update_text(&format!("{}", self));

        node.clone()
    }
}

impl<G: GenericNode> Render<G> for TemplateList<G> {
    fn render(&self) -> G {
        let fragment = G::fragment();

        for item in self
            .templates
            .inner_signal()
            .get()
            .borrow()
            .clone()
            .into_iter()
        {
            fragment.append_render(Box::new(move || {
                let item = item.clone();
                Box::new(item)
            }));
        }

        fragment
    }

    fn update_node(&self, parent: &G, node: &G) -> G {
        let templates = self.templates.inner_signal().get(); // subscribe to templates
        let changes = Rc::clone(&self.templates.changes());

        for change in changes.borrow().iter() {
            match change {
                VecDiff::Replace { values } => {
                    let first = templates.borrow().first().map(|x| x.node.clone());

                    for value in values {
                        parent.insert_child_before(&value.node, first.as_ref());
                    }

                    for template in templates.borrow().iter() {
                        parent.remove_child(&template.node);
                    }
                }
                VecDiff::Insert { index, value } => {
                    parent.insert_child_before(
                        &value.node,
                        templates
                            .borrow()
                            .get(*index)
                            .map(|template| template.node.next_sibling())
                            .flatten()
                            .as_ref(),
                    );
                }
                VecDiff::Update { index, value } => {
                    parent.replace_child(&templates.borrow()[*index].node, &value.node);
                }
                VecDiff::Remove { index } => {
                    parent.remove_child(&templates.borrow()[*index].node);
                }
                VecDiff::Swap { index1, index2 } => {
                    let child1 = &templates.borrow()[*index1].node;
                    let child2 = &templates.borrow()[*index2].node;
                    parent.replace_child(child1, child2);
                    parent.replace_child(child2, child1);
                }
                VecDiff::Push { value } => {
                    parent.insert_child_before(
                        &value.node,
                        templates
                            .borrow()
                            .last()
                            .map(|last| last.node.next_sibling())
                            .flatten()
                            .as_ref(),
                    );
                }
                VecDiff::Pop => {
                    if let Some(last) = templates.borrow().last() {
                        parent.remove_child(&last.node);
                    }
                }
                VecDiff::Clear => {
                    for template in templates.borrow().iter() {
                        parent.remove_child(&template.node);
                    }
                }
            }
        }

        node.clone()
    }
}

impl<G: GenericNode> Render<G> for TemplateResult<G> {
    fn render(&self) -> G {
        self.node.clone()
    }
}

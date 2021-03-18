//! Trait for describing how something should be rendered into DOM nodes.

use std::fmt;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{Node, Text};

use crate::reactive::VecDiff;
use crate::TemplateResult;
use crate::{internal::*, TemplateList};

/// Trait for describing how something should be rendered into DOM nodes.
pub trait Render {
    /// Called during the initial render when creating the DOM nodes. Should return a [`Node`].
    fn render(&self) -> Node;

    /// Called when the node should be updated with new state.
    /// The default implementation of this will replace the child node completely with the result of calling `render` again.
    /// Another implementation might be better suited to some specific types.
    /// For example, text nodes can simply replace the inner text instead of recreating a new node.
    ///
    /// Returns the new node. If the node is reused instead of replaced, the returned node is simply the node passed in.
    fn update_node(&self, parent: &Node, node: &Node) -> Node {
        let new_node = self.render();
        parent.replace_child(&new_node, &node).unwrap();
        new_node
    }
}

impl<T: fmt::Display + ?Sized> Render for T {
    fn render(&self) -> Node {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_text_node(&format!("{}", self))
            .into()
    }

    fn update_node(&self, _parent: &Node, node: &Node) -> Node {
        // replace `textContent` of `node` instead of recreating
        node.clone()
            .dyn_into::<Text>()
            .unwrap()
            .set_text_content(Some(&format!("{}", self)));

        node.clone()
    }
}

impl Render for TemplateList {
    fn render(&self) -> Node {
        let fragment = fragment();

        for item in self
            .templates
            .inner_signal()
            .get()
            .borrow()
            .clone()
            .into_iter()
        {
            append_render(
                &fragment,
                Box::new(move || {
                    let item = item.clone();
                    Box::new(item)
                }),
            );
        }

        fragment.into()
    }

    fn update_node(&self, parent: &Node, node: &Node) -> Node {
        let templates = self.templates.inner_signal().get(); // subscribe to templates
        let changes = Rc::clone(&self.templates.changes());

        for change in changes.borrow().iter() {
            match change {
                VecDiff::Replace { values } => {
                    let first = templates.borrow().first().map(|x| x.node.clone());

                    for value in values {
                        parent.insert_before(&value.node, first.as_ref()).unwrap();
                    }

                    for template in templates.borrow().iter() {
                        parent.remove_child(&template.node).unwrap();
                    }
                }
                VecDiff::Insert { index, value } => {
                    parent
                        .insert_before(
                            &value.node,
                            templates
                                .borrow()
                                .get(*index)
                                .map(|template| template.node.next_sibling())
                                .flatten()
                                .as_ref(),
                        )
                        .unwrap();
                }
                VecDiff::Update { index, value } => {
                    parent
                        .replace_child(&templates.borrow()[*index].node, &value.node)
                        .unwrap();
                }
                VecDiff::Remove { index } => {
                    parent
                        .remove_child(&templates.borrow()[*index].node)
                        .unwrap();
                }
                VecDiff::Swap { index1, index2 } => {
                    let child1 = &templates.borrow()[*index1].node;
                    let child2 = &templates.borrow()[*index2].node;
                    parent.replace_child(child1, child2).unwrap();
                    parent.replace_child(child2, child1).unwrap();
                }
                VecDiff::Push { value } => {
                    parent
                        .insert_before(
                            &value.node,
                            templates
                                .borrow()
                                .last()
                                .map(|last| last.node.next_sibling())
                                .flatten()
                                .as_ref(),
                        )
                        .unwrap();
                }
                VecDiff::Pop => {
                    if let Some(last) = templates.borrow().last() {
                        parent.remove_child(&last.node).unwrap();
                    }
                }
                VecDiff::Clear => {
                    for template in templates.borrow().iter() {
                        parent.remove_child(&template.node).unwrap();
                    }
                }
            }
        }

        node.clone()
    }
}

impl Render for TemplateResult {
    fn render(&self) -> Node {
        self.node.clone()
    }
}

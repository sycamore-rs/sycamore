use std::fmt;

use wasm_bindgen::JsCast;
use web_sys::{Element, Node, Text};

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

        for item in self.templates.clone().into_iter() {
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
        while let Some(child) = parent.last_child() {
            child.dyn_into::<Element>().unwrap().remove();
        }

        for item in self.templates.clone().into_iter() {
            parent.append_child(&item.render()).unwrap();
        }

        node.clone()
    }
}

impl Render for TemplateResult {
    fn render(&self) -> Node {
        self.node.clone()
    }
}

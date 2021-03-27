use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Node};

use crate::prelude::*;

pub trait GenericNode: Debug + Clone + PartialEq + Eq + 'static {
    fn element(tag: &str) -> Self;
    fn text_node(text: &str) -> Self;
    fn fragment() -> Self;
    fn marker() -> Self;
    
    fn append_child(&self, child: &Self);
    fn insert_before_self(&self, new_node: &Self);

    #[deprecated]
    fn insert_node_before(&self, newNode: &Self, referenceNode: Option<&Self>);
    fn remove_child(&self, child: &Self);
    fn remove_self(&self);
    fn replace_child(&self, old: &Self, new: &Self);
    fn append_render(&self, child: Box<dyn Fn() -> Box<dyn Render<Self>>>) {
        let parent = self.clone();

        let node = create_effect_initial(cloned!((parent) => move || {
            let node = RefCell::new(child().render());

            let effect = cloned!((node) => move || {
                let new_node = child().update_node(&parent, &node.borrow());
                *node.borrow_mut() = new_node;
            });

            (Rc::new(effect), node)
        }));

        parent.append_child(&node.borrow());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomNode {
    node: Node,
}

impl DomNode {
    pub fn inner_element(&self) -> Node {
        self.node.clone()
    }
}

impl GenericNode for DomNode {
    fn element(tag: &str) -> Self {
        DomNode {
            node: web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element(tag)
                .unwrap()
                .dyn_into()
                .unwrap(),
        }
    }

    fn text_node(text: &str) -> Self {
        DomNode {
            node: web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_text_node(text)
                .into(),
        }
    }

    fn fragment() -> Self {
        DomNode {
            node: web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_document_fragment()
                .into(),
        }
    }

    fn marker() -> Self {
        DomNode {
            node: web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_comment("")
                .into(),
        }
    }

    fn append_child(&self, child: &Self) {
        self.node.append_child(&child.node).unwrap();
    }

    fn insert_before_self(&self, new_node: &Self) {}

    fn insert_node_before(&self, newNode: &Self, referenceNode: Option<&Self>) {
        todo!()
    }

    fn remove_child(&self, child: &Self) {
        unimplemented!()
    }

    fn remove_self(&self) {
        self.node.unchecked_ref::<HtmlElement>().remove();
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        unimplemented!()
    }
}

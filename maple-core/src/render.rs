use std::fmt;

use web_sys::Node;

use crate::TemplateResult;

pub trait Render {
    fn render(&self) -> Node;
}

impl<T: fmt::Display> Render for T {
    fn render(&self) -> Node {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_text_node(&format!("{}", self))
            .into()
    }
}

impl Render for TemplateResult {
    fn render(&self) -> Node {
        self.node.clone()
    }
}

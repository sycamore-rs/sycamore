use std::fmt;

use web_sys::Node;

use crate::TemplateResult;
use crate::{internal::*, TemplateList};

pub trait Render {
    fn render(&mut self) -> Node;
}

impl<T: fmt::Display + ?Sized> Render for T {
    fn render(&mut self) -> Node {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_text_node(&format!("{}", self))
            .into()
    }
}

impl Render for TemplateList {
    fn render(&mut self) -> Node {
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
}

impl Render for TemplateResult {
    fn render(&mut self) -> Node {
        self.node.clone()
    }
}

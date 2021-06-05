use crate::template::{visit_element, Element, HtmlTree, HtmlVisit};

static VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr", "command", "keygen", "menuitem",
];

#[derive(Debug, Default)]
pub struct TemplateVisitor {
    buf: String,
    elem_count: u32,
}

impl TemplateVisitor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'ast> HtmlVisit<'ast> for TemplateVisitor {
    fn visit_element(&mut self, node: &'ast Element) {
        let tag_str = node.tag_name.to_string();
        self.buf += &format!("<{}>", tag_str); // TODO: static attributes
        visit_element(self, node);
        self.buf += &format!("</{}>", tag_str);
        self.elem_count += 1;
    }
}

/// Generates a string to be used for creating a `<template>` element.
pub fn gen_template_string(tree: &HtmlTree) -> String {
    let mut template_visitor = TemplateVisitor::new();

    template_visitor.visit_html_tree(tree);

    template_visitor.buf
}

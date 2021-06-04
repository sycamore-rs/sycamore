use crate::template::*;

/// Visitor for HTML nodes.
pub trait HtmlVisit<'ast> {
    fn visit_html_tree(&mut self, node: &'ast HtmlTree) {
        visit_html_tree(self, node);
    }

    fn visit_component(&mut self, node: &'ast Component) {
        visit_component(self, node);
    }

    fn visit_element(&mut self, node: &'ast Element) {
        visit_element(self, node);
    }

    fn visit_tag_name(&mut self, node: &'ast TagName) {
        visit_tag_name(self, node);
    }

    fn visit_attribute(&mut self, node: &'ast Attribute) {
        visit_attribute(self, node);
    }

    fn visit_text(&mut self, node: &'ast Text) {
        visit_text(self, node);
    }
}

fn visit_html_tree<'ast, V>(v: &mut V, node: &'ast HtmlTree)
where
    V: HtmlVisit<'ast> + ?Sized,
{
    match node {
        HtmlTree::Component(component) => v.visit_component(component),
        HtmlTree::Element(element) => v.visit_element(element),
        HtmlTree::Text(text) => todo!(),
    }
}

fn visit_component<'ast, V>(_v: &mut V, _node: &'ast Component)
where
    V: HtmlVisit<'ast> + ?Sized,
{
    // noop
}

pub fn visit_element<'ast, V>(v: &mut V, node: &'ast Element)
where
    V: HtmlVisit<'ast> + ?Sized,
{
    v.visit_tag_name(&node.tag_name);
    if let Some(attribute_list) = &node.attributes {
        for attribute in &attribute_list.attributes {
            v.visit_attribute(attribute);
        }
    }
    if let Some(children) = &node.children {
        for child in &children.body {
            v.visit_html_tree(child);
        }
    }
}

fn visit_tag_name<'ast, V>(_v: &mut V, _node: &'ast TagName)
where
    V: HtmlVisit<'ast> + ?Sized,
{
    // noop
}

fn visit_attribute<'ast, V>(_v: &mut V, _node: &'ast Attribute)
where
    V: HtmlVisit<'ast> + ?Sized,
{
    // noop
}

fn visit_text<'ast, V>(v: &mut V, node: &'ast Text)
where
    V: HtmlVisit<'ast> + ?Sized,
{
    // noop
}

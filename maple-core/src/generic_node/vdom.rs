use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::generic_node::{EventListener, GenericNode};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Node {
    Element(Rc<RefCell<Element>>),
    Comment(Rc<RefCell<Comment>>),
    Text(Rc<RefCell<Text>>),
    Fragment(Rc<RefCell<Fragment>>),
}

impl Node {
    fn unwrap_element(&self) -> &Rc<RefCell<Element>> {
        match self {
            Node::Element(e) => e,
            _ => panic!("The node is not an element"),
        }
    }
    fn unwrap_text(&self) -> &Rc<RefCell<Text>> {
        match self {
            Node::Text(e) => e,
            _ => panic!("The node is not a text node"),
        }
    }
}

impl GenericNode for Node {
    fn element(tag: &str) -> Self {
        Node::Element(Rc::new(RefCell::new(Element {
            name: tag.to_string(),
            attributes: Default::default(),
            children: Default::default(),
        })))
    }

    fn text_node(text: &str) -> Self {
        Node::Text(Rc::new(RefCell::new(Text(text.to_string()))))
    }

    fn fragment() -> Self {
        Node::Fragment(Default::default())
    }

    fn empty() -> Self {
        Node::Comment(Default::default())
    }

    fn set_attribute(&self, name: &str, value: &str) {
        self.unwrap_element()
            .borrow_mut()
            .attributes
            .insert(name.to_string(), value.to_string());
    }

    fn append_child(&self, child: &Self) {
        self.unwrap_element()
            .borrow_mut()
            .children
            .0
            .push(child.clone());
    }

    fn insert_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        let mut ele = self.unwrap_element().borrow_mut();
        let children = &mut ele.children.0;
        match reference_node {
            None => self.append_child(new_node),
            Some(reference) => {
                children.insert(
                    children
                        .iter()
                        .enumerate()
                        .find_map(|(i, child)| (child == reference).then(|| i))
                        .expect("Couldn't find reference node"),
                    reference.clone(),
                );
            }
        }
    }

    fn remove_child(&self, child: &Self) {
        let mut ele = self.unwrap_element().borrow_mut();
        let index = ele.children
            .0
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == child).then(|| i))
            .expect("Couldn't find child");
        ele.children.0.remove(
            index,
        );
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        let mut ele = self.unwrap_element().borrow_mut();
        let children = &mut ele.children.0;
        let index = children
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == old).then(|| i))
            .expect("Couldn't find child");
        children[index] = new.clone();
    }

    fn insert_sibling_before(&self, _child: &Self) {
        unimplemented!()
    }

    fn parent_node(&self) -> Option<Self> {
        unimplemented!()
    }

    fn next_sibling(&self) -> Option<Self> {
        unimplemented!()
    }

    fn remove_self(&self) {
        unimplemented!()
    }

    fn event(&self, _name: &str, _handler: Box<EventListener>) {
        // Don't do anything
    }

    fn update_text(&self, text: &str) {
        self.unwrap_text().borrow_mut().0 = text.to_string();
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Element(x) => write!(f, "{}", x.borrow()),
            Node::Comment(x) => write!(f, "{}", x.borrow()),
            Node::Text(x) => write!(f, "{}", x.borrow()),
            Node::Fragment(x) => write!(f, "{}", x.borrow()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Element {
    name: String,
    attributes: HashMap<String, String>,
    children: Fragment,
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}", self.name)?;
        for (name, value) in &self.attributes {
            write!(
                f,
                r#" {}="{}""#,
                name,
                html_escape::encode_double_quoted_attribute(value)
            )?;
        }
        write!(f, ">{}</{}>", self.children, self.name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Comment(String);

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<!--{}-->", self.0.replace("-->", "--&gt;"))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Text(String);

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", html_escape::encode_text_minimal(&self.0))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Fragment(Vec<Node>);

impl fmt::Display for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in &self.0 {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}

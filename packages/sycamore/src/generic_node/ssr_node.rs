//! Rendering backend for Server Side Rendering, aka. SSR.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use std::{fmt, mem};

use wasm_bindgen::prelude::*;

use crate::generic_node::{EventListener, GenericNode};
use crate::rx::create_root;
use crate::template::Template;

static VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr", "command", "keygen", "menuitem",
];

/// Inner representation for [`SsrNode`].
#[derive(Debug)]
enum SsrNodeType {
    Element(RefCell<Element>),
    Comment(RefCell<Comment>),
    Text(RefCell<Text>),
}

#[derive(Debug, Clone)]
struct SsrNodeInner {
    ty: Rc<SsrNodeType>,
    /// No parent if `Weak::upgrade` returns `None`.
    parent: RefCell<Weak<SsrNodeInner>>,
}

/// Rendering backend for Server Side Rendering, aka. SSR.
///
/// _This API requires the following crate features to be activated: `ssr`_
#[derive(Debug, Clone)]
pub struct SsrNode(Rc<SsrNodeInner>);

impl PartialEq for SsrNode {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0.ty, &other.0.ty)
    }
}

impl Eq for SsrNode {}

impl Hash for SsrNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl SsrNode {
    fn new(ty: SsrNodeType) -> Self {
        Self(Rc::new(SsrNodeInner {
            ty: Rc::new(ty),
            parent: RefCell::new(Weak::new()), // no parent
        }))
    }

    fn set_parent(&self, parent: Weak<SsrNodeInner>) {
        if let Some(old_parent) = self.parent_node() {
            old_parent.try_remove_child(self);
        }

        *self.0.parent.borrow_mut() = parent;
    }

    #[track_caller]
    pub fn unwrap_element(&self) -> &RefCell<Element> {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => e,
            _ => panic!("node is not an element"),
        }
    }

    #[track_caller]
    pub fn unwrap_text(&self) -> &RefCell<Text> {
        match &self.0.ty.as_ref() {
            SsrNodeType::Text(e) => e,
            _ => panic!("node is not a text node"),
        }
    }

    fn try_remove_child(&self, child: &Self) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => {
                let children = e
                    .borrow()
                    .children
                    .clone()
                    .into_iter()
                    .filter(|node| node == child)
                    .collect();
                e.borrow_mut().children = children;
            }
            _ => panic!("node type cannot have children"),
        }
    }
}

impl GenericNode for SsrNode {
    fn element(tag: &str) -> Self {
        SsrNode::new(SsrNodeType::Element(RefCell::new(Element {
            name: tag.to_string(),
            attributes: HashMap::new(),
            children: Default::default(),
        })))
    }

    fn text_node(text: &str) -> Self {
        SsrNode::new(SsrNodeType::Text(RefCell::new(Text(text.to_string()))))
    }

    fn marker() -> Self {
        SsrNode::new(SsrNodeType::Comment(Default::default()))
    }

    fn set_attribute(&self, name: &str, value: &str) {
        self.unwrap_element()
            .borrow_mut()
            .attributes
            .insert(name.to_string(), value.to_string());
    }

    fn set_property(&self, _name: &str, _value: &JsValue) {
        // Noop.
    }

    fn append_child(&self, child: &Self) {
        child.set_parent(Rc::downgrade(&self.0));

        match self.0.ty.as_ref() {
            SsrNodeType::Element(element) => element.borrow_mut().children.push(child.clone()),
            _ => panic!("node type cannot have children"),
        }
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        if let Some(reference_node) = reference_node {
            debug_assert_eq!(
                reference_node.parent_node().as_ref(),
                Some(self),
                "reference node is not a child of this node"
            );
        }

        new_node.set_parent(Rc::downgrade(&self.0));

        let mut children = match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => mem::take(&mut e.borrow_mut().children),
            _ => panic!("node type cannot have children"),
        };

        match reference_node {
            None => self.append_child(new_node),
            Some(reference) => {
                children.insert(
                    children
                        .iter()
                        .enumerate()
                        .find_map(|(i, child)| (child == reference).then(|| i))
                        .expect("couldn't find reference node"),
                    new_node.clone(),
                );
            }
        }

        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => e.borrow_mut().children = children,
            _ => panic!("node type cannot have children"),
        };
    }

    fn remove_child(&self, child: &Self) {
        let mut children = match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => mem::take(&mut e.borrow_mut().children),
            _ => panic!("node type cannot have children"),
        };

        let index = children
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == child).then(|| i))
            .expect("couldn't find child");
        children.remove(index);

        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => e.borrow_mut().children = children,
            _ => panic!("node type cannot have children"),
        };
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        new.set_parent(Rc::downgrade(&self.0));

        let mut ele = self.unwrap_element().borrow_mut();
        let children = &mut ele.children;
        let index = children
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == old).then(|| i))
            .expect("Couldn't find child");
        children[index] = new.clone();
    }

    fn insert_sibling_before(&self, child: &Self) {
        child.set_parent(Rc::downgrade(
            &self.parent_node().expect("no parent for this node").0,
        ));

        self.parent_node()
            .unwrap()
            .insert_child_before(child, Some(self));
    }

    fn parent_node(&self) -> Option<Self> {
        self.0.parent.borrow().upgrade().map(SsrNode)
    }

    fn next_sibling(&self) -> Option<Self> {
        unimplemented!()
    }

    fn remove_self(&self) {
        unimplemented!()
    }

    fn event(&self, _name: &str, _handler: Box<EventListener>) {
        // Noop. Events are attached on client side.
    }

    fn update_inner_text(&self, text: &str) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(el) => el.borrow_mut().children = vec![SsrNode::text_node(text)],
            SsrNodeType::Comment(_c) => panic!("cannot update inner text on comment node"),
            SsrNodeType::Text(t) => t.borrow_mut().0 = text.to_string(),
        }
    }
}

impl fmt::Display for SsrNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(x) => write!(f, "{}", x.borrow()),
            SsrNodeType::Comment(x) => write!(f, "{}", x.borrow()),
            SsrNodeType::Text(x) => write!(f, "{}", x.borrow()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Element {
    name: String,
    attributes: HashMap<String, String>,
    children: Vec<SsrNode>,
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

        // Check if self-closing tag (void-element).
        if self.children.is_empty() && VOID_ELEMENTS.iter().any(|tag| tag == &self.name) {
            write!(f, " />")?;
        } else {
            write!(f, ">")?;
            for child in &self.children {
                write!(f, "{}", child)?;
            }
            write!(f, "</{}>", self.name)?;
        }
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

/// Render a [`Template`] into a static [`String`]. Useful
/// for rendering to a string on the server side.
///
/// _This API requires the following crate features to be activated: `ssr`_
pub fn render_to_string(template: impl FnOnce() -> Template<SsrNode>) -> String {
    let mut ret = String::new();
    let _scope = create_root(|| {
        for node in template().flatten() {
            ret.push_str(&format!("{}", node));
        }
    });

    ret
}

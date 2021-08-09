//! Rendering backend for Server Side Rendering, aka. SSR.

use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

use ahash::AHashMap;
use wasm_bindgen::prelude::*;

use crate::generic_node::{EventHandler, GenericNode};
use crate::reactive::create_root;
use crate::template::Template;

static VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr", "command", "keygen", "menuitem",
];

/// Inner representation for [`SsrNode`].
#[derive(Debug, Clone)]
enum SsrNodeType {
    Element(RefCell<Element>),
    Comment(RefCell<Comment>),
    Text(RefCell<Text>),
    RawText(RefCell<RawText>),
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

    /// Create a new raw text node.
    ///
    /// Do not pass unsanitized user input to this function. When the node is rendered, no escaping
    /// will be performed which might lead to a XSS (Cross Site Scripting) attack.
    pub fn raw_text_node(html: &str) -> Self {
        SsrNode::new(SsrNodeType::RawText(RefCell::new(RawText(
            html.to_string(),
        ))))
    }
}

impl GenericNode for SsrNode {
    fn element(tag: &str) -> Self {
        SsrNode::new(SsrNodeType::Element(RefCell::new(Element {
            name: tag.to_string(),
            attributes: AHashMap::new(),
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

    fn set_class_name(&self, value: &str) {
        self.set_attribute("class", value);
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

    fn first_child(&self) -> Option<Self> {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(element) => element.borrow_mut().children.first().cloned(),
            _ => panic!("node type cannot have children"),
        }
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        new_node.set_parent(Rc::downgrade(&self.0));

        match reference_node {
            None => self.append_child(new_node),
            Some(reference) => {
                match self.0.ty.as_ref() {
                    SsrNodeType::Element(e) => {
                        let children = &mut e.borrow_mut().children;
                        let index = children
                            .iter()
                            .enumerate()
                            .find_map(|(i, child)| (child == reference).then(|| i))
                            .expect("reference node is not a child of this node");
                        children.insert(index, new_node.clone());
                    }
                    _ => panic!("node type cannot have children"),
                };
            }
        }
    }

    fn remove_child(&self, child: &Self) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => {
                child.set_parent(Weak::new());
                let index = e
                    .borrow()
                    .children
                    .iter()
                    .enumerate()
                    .find_map(|(i, c)| (c == child).then(|| i))
                    .expect("the node to be removed is not a child of this node");
                e.borrow_mut().children.remove(index);
            }
            _ => panic!("node type cannot have children"),
        }
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        new.set_parent(Rc::downgrade(&self.0));

        let mut ele = self.unwrap_element().borrow_mut();
        let children = &mut ele.children;
        let index = children
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == old).then(|| i))
            .expect("the node to be replaced is not a child of this node");
        children[index].set_parent(Weak::new());
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
        let parent = self.parent_node().expect("node must have a parent");
        match parent.0.ty.as_ref() {
            SsrNodeType::Element(e) => {
                let children = &e.borrow().children;
                children
                    .iter()
                    .skip_while(|child| *child != self)
                    .skip(1)
                    .take(1)
                    .cloned()
                    .next()
            }
            _ => panic!("node type cannot have children"),
        }
    }

    fn remove_self(&self) {
        self.parent_node()
            .expect("node must have a parent")
            .remove_child(self);
    }

    fn event(&self, _name: &str, _handler: Box<EventHandler>) {
        // Noop. Events are attached on client side.
    }

    fn update_inner_text(&self, text: &str) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(el) => el.borrow_mut().children = vec![SsrNode::text_node(text)],
            SsrNodeType::Comment(_c) => panic!("cannot update inner text on comment node"),
            SsrNodeType::Text(t) => t.borrow_mut().0 = text.to_string(),
            SsrNodeType::RawText(_t) => panic!("cannot update inner text on raw text node"),
        }
    }

    fn dangerously_set_inner_html(&self, html: &str) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(el) => {
                el.borrow_mut().children = vec![SsrNode::raw_text_node(html)];
            }
            SsrNodeType::Comment(_c) => panic!("cannot update inner text on comment node"),
            SsrNodeType::Text(_t) => panic!("cannot update inner text on text node"),
            SsrNodeType::RawText(t) => t.borrow_mut().0 = html.to_string(),
        }
    }

    fn clone_node(&self) -> Self {
        let inner = SsrNodeInner {
            ty: Rc::new(self.0.ty.as_ref().clone()),
            parent: RefCell::new(Weak::new()),
        };
        Self(Rc::new(inner))
    }
}

impl fmt::Display for SsrNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(x) => write!(f, "{}", x.borrow()),
            SsrNodeType::Comment(x) => write!(f, "{}", x.borrow()),
            SsrNodeType::Text(x) => write!(f, "{}", x.borrow()),
            SsrNodeType::RawText(x) => write!(f, "{}", x.borrow()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Element {
    name: String,
    attributes: AHashMap<String, String>,
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

/// Un-escaped text node.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct RawText(String);

impl fmt::Display for RawText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
            ret.push_str(&node.to_string());
        }
    });

    ret
}

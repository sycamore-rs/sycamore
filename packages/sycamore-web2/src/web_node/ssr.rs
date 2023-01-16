//! Internal implementation for SSR.

use std::borrow::Cow;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

use indexmap::map::IndexMap;
use sycamore_reactive::Scope;
use wasm_bindgen::JsValue;

use crate::VOID_ELEMENTS;

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

    /// Create a new element without hydration keys or another other special attributes.
    fn new_element_raw(
        tag: Cow<'static, str>,
        attributes: IndexMap<Cow<'static, str>, Cow<'static, str>>,
        children: Vec<Self>,
    ) -> Self {
        Self::new(SsrNodeType::Element(RefCell::new(Element {
            name: tag,
            attributes,
            children,
        })))
    }

    fn set_parent(&self, parent: Weak<SsrNodeInner>) {
        if let Some(old_parent) = self.parent_node() {
            old_parent.try_remove_child(self);
        }

        *self.0.parent.borrow_mut() = parent;
    }

    /// Get an [`Element`], or `panic!` if wrong type.
    #[track_caller]
    fn unwrap_element(&self) -> &RefCell<Element> {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => e,
            _ => panic!("node is not an element"),
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
                    .filter(|node| node != child)
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
    pub fn raw_text_node(html: Cow<'static, str>) -> Self {
        SsrNode::new(SsrNodeType::RawText(RefCell::new(RawText(html))))
    }
}

/// `GenericNode` methods.
impl SsrNode {
    pub fn text_node(text: Cow<'static, str>) -> Self {
        Self::new(SsrNodeType::Text(RefCell::new(Text(text))))
    }

    pub fn marker_with_text(text: Cow<'static, str>) -> Self {
        Self::new(SsrNodeType::Comment(RefCell::new(Comment(text))))
    }

    pub fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>) {
        self.unwrap_element()
            .borrow_mut()
            .attributes
            .insert(name, value);
    }

    pub fn remove_attribute(&self, name: Cow<'static, str>) {
        self.unwrap_element().borrow_mut().attributes.remove(&name);
    }

    pub fn set_class_name(&self, value: Cow<'static, str>) {
        self.set_attribute("class".into(), value);
    }

    pub fn append_child(&self, child: &Self) {
        child.set_parent(Rc::downgrade(&self.0));

        match self.0.ty.as_ref() {
            SsrNodeType::Element(element) => element.borrow_mut().children.push(child.clone()),
            _ => panic!("node type cannot have children"),
        }
    }

    pub fn first_child(&self) -> Option<Self> {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(element) => element.borrow_mut().children.first().cloned(),
            _ => panic!("node type cannot have children"),
        }
    }

    pub fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
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
                            .find_map(|(i, child)| (child == reference).then_some(i))
                            .expect("reference node is not a child of this node");
                        children.insert(index, new_node.clone());
                    }
                    _ => panic!("node type cannot have children"),
                };
            }
        }
    }

    pub fn remove_child(&self, child: &Self) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => {
                let initial_children_len = e.borrow().children.len();
                if child.parent_node().as_ref() != Some(self) {
                    panic!("the node to be removed is not a child of this node");
                }
                child.set_parent(Weak::new());
                debug_assert_eq!(e.borrow().children.len(), initial_children_len - 1);
            }
            _ => panic!("node type cannot have children"),
        }
    }

    pub fn replace_child(&self, old: &Self, new: &Self) {
        new.set_parent(Rc::downgrade(&self.0));

        let mut ele = self.unwrap_element().borrow_mut();
        let children = &mut ele.children;
        let index = children
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == old).then_some(i))
            .expect("the node to be replaced is not a child of this node");
        *children[index].0.parent.borrow_mut() = Weak::new();
        children[index] = new.clone();
    }

    pub fn insert_sibling_before(&self, child: &Self) {
        child.set_parent(Rc::downgrade(
            &self.parent_node().expect("no parent for this node").0,
        ));

        self.parent_node()
            .unwrap()
            .insert_child_before(child, Some(self));
    }

    pub fn parent_node(&self) -> Option<Self> {
        self.0.parent.borrow().upgrade().map(SsrNode)
    }

    pub fn next_sibling(&self) -> Option<Self> {
        let parent = self.parent_node().expect("node must have a parent");
        match parent.0.ty.as_ref() {
            SsrNodeType::Element(e) => {
                let children = &e.borrow().children;
                children
                    .iter()
                    .skip_while(|child| *child != self)
                    .nth(1)
                    .cloned()
            }
            _ => panic!("node type cannot have children"),
        }
    }

    pub fn remove_self(&self) {
        self.parent_node()
            .expect("node must have a parent")
            .remove_child(self);
    }

    pub fn update_inner_text(&self, text: Cow<'static, str>) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(el) => el.borrow_mut().children = vec![SsrNode::text_node(text)],
            SsrNodeType::Comment(_c) => panic!("cannot update inner text on comment node"),
            SsrNodeType::Text(t) => t.borrow_mut().0 = text,
            SsrNodeType::RawText(_t) => panic!("cannot update inner text on raw text node"),
        }
    }

    pub fn dangerously_set_inner_html(&self, html: Cow<'static, str>) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(el) => {
                el.borrow_mut().children = vec![SsrNode::raw_text_node(html)];
            }
            SsrNodeType::Comment(_c) => panic!("cannot update inner text on comment node"),
            SsrNodeType::Text(_t) => panic!("cannot update inner text on text node"),
            SsrNodeType::RawText(t) => t.borrow_mut().0 = html,
        }
    }

    pub fn clone_node(&self) -> Self {
        let inner = SsrNodeInner {
            ty: Rc::new(self.0.ty.as_ref().clone()),
            parent: RefCell::new(Weak::new()),
        };
        Self(Rc::new(inner))
    }
}

/// `GenericNodeElements` methods.
impl SsrNode {
    pub fn element_from_tag(tag: Cow<'static, str>) -> Self {
        Self::new_element_raw(tag, IndexMap::default(), Vec::new())
    }

    pub fn element_from_tag_namespace(
        tag: Cow<'static, str>,
        _namespace: Cow<'static, str>,
    ) -> Self {
        Self::element_from_tag(tag)
    }

    pub fn add_event_listener<'a>(&self, _: Scope<'a>, _: &str, _: Box<dyn FnMut(JsValue) + 'a>) {
        // no-op. Events do nothing on the server-side.
    }
}

impl SsrNode {
    pub fn set_property(&self, _name: &str, _value: JsValue) {
        // no-op. Properties do nothing on the server-side.
    }
}

/// Write the [`SsrNode`] to a string buffer.
/// Implementation details.
#[doc(hidden)]
pub trait WriteToString {
    fn write_to_string(&self, s: &mut String);
}

impl WriteToString for SsrNode {
    fn write_to_string(&self, s: &mut String) {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(x) => x.borrow().write_to_string(s),
            SsrNodeType::Comment(x) => x.borrow().write_to_string(s),
            SsrNodeType::Text(x) => x.borrow().write_to_string(s),
            SsrNodeType::RawText(x) => x.borrow().write_to_string(s),
        }
    }
}

/// A SSR element.
#[derive(Debug, Clone, Eq, PartialEq)]
struct Element {
    name: Cow<'static, str>,
    attributes: IndexMap<Cow<'static, str>, Cow<'static, str>>,
    children: Vec<SsrNode>,
}

impl WriteToString for Element {
    fn write_to_string(&self, s: &mut String) {
        s.reserve("<".len() + self.name.len());
        s.push('<');
        s.push_str(&self.name);
        for (name, value) in &self.attributes {
            let value_escaped = html_escape::encode_double_quoted_attribute(value);
            s.reserve(" ".len() + name.len() + "=\"".len() + value_escaped.len() + "\"".len());
            s.push(' ');
            s.push_str(name);
            s.push_str("=\"");
            s.push_str(&value_escaped);
            s.push('"');
        }

        // Check if self-closing tag (void-element).
        if self.children.is_empty() && VOID_ELEMENTS.contains(&*self.name) {
            s.push_str("/>");
        } else {
            s.push('>');
            for child in &self.children {
                child.write_to_string(s);
            }
            s.reserve("</".len() + self.name.len() + ">".len());
            s.push_str("</");
            s.push_str(&self.name);
            s.push('>');
        }
    }
}

/// A SSR comment node.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct Comment(Cow<'static, str>);

impl WriteToString for Comment {
    fn write_to_string(&self, s: &mut String) {
        let escaped = self.0.replace("-->", "--&gt;");
        s.reserve("<!--".len() + escaped.len() + "-->".len());
        s.push_str("<!--");
        s.push_str(&escaped);
        s.push_str("-->");
    }
}

/// A SSR text node.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct Text(Cow<'static, str>);

impl WriteToString for Text {
    fn write_to_string(&self, s: &mut String) {
        s.push_str(&html_escape::encode_text_minimal(&self.0));
    }
}

/// Un-escaped text node.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct RawText(Cow<'static, str>);

impl WriteToString for RawText {
    fn write_to_string(&self, s: &mut String) {
        s.push_str(&self.0);
    }
}

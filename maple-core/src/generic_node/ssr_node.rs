use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::{fmt, mem};

use crate::generic_node::{EventListener, GenericNode};

/// Rendering backend for Server Side Rendering, aka. SSR.
///
/// _This API requires the following crate features to be activated: `ssr`_
#[derive(Debug)]
enum SsrNodeType {
    Element(RefCell<Element>),
    Comment(RefCell<Comment>),
    Text(RefCell<Text>),
    Fragment(RefCell<Fragment>),
}

#[derive(Debug, Clone)]
struct SsrNodeInner {
    ty: Rc<SsrNodeType>,
    /// No parent if `Weak::upgrade` returns `None`.
    parent: RefCell<Weak<SsrNodeInner>>,
}

#[derive(Debug, Clone)]
pub struct SsrNode(Rc<SsrNodeInner>);

impl PartialEq for SsrNode {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0.ty, &other.0.ty)
    }
}

impl Eq for SsrNode {}

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
    fn unwrap_element(&self) -> &RefCell<Element> {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => e,
            _ => panic!("node is not an element"),
        }
    }

    #[track_caller]
    fn unwrap_text(&self) -> &RefCell<Text> {
        match &self.0.ty.as_ref() {
            SsrNodeType::Text(e) => e,
            _ => panic!("node is not a text node"),
        }
    }

    // FIXME: recursively visit Fragments and call try_remove_child
    fn try_remove_child(&self, child: &Self) {
        let mut children = match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => mem::take(&mut e.borrow_mut().children.0),
            SsrNodeType::Fragment(f) => mem::take(&mut f.borrow_mut().0),
            _ => panic!("node type cannot have children"),
        };

        if let Some(index) = children
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == child).then(|| i))
        {
            children.remove(index);
        } else {
            // try remove from child Fragments
            for c in &children {
                if let SsrNodeType::Fragment(fragment) = c.0.ty.as_ref() {
                    for c in &fragment.borrow().0 {
                        c.try_remove_child(&child);
                    }
                }
            }
        }

        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => e.borrow_mut().children.0 = children,
            SsrNodeType::Fragment(f) => f.borrow_mut().0 = children,
            _ => panic!("node type cannot have children"),
        };
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

    fn fragment() -> Self {
        SsrNode::new(SsrNodeType::Fragment(Default::default()))
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

    fn append_child(&self, child: &Self) {
        child.set_parent(Rc::downgrade(&self.0));

        match self.0.ty.as_ref() {
            SsrNodeType::Element(element) => element.borrow_mut().children.0.push(child.clone()),
            SsrNodeType::Fragment(fragment) => fragment.borrow_mut().0.push(child.clone()),
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
            SsrNodeType::Element(e) => mem::take(&mut e.borrow_mut().children.0),
            SsrNodeType::Fragment(f) => mem::take(&mut f.borrow_mut().0),
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
            SsrNodeType::Element(e) => e.borrow_mut().children.0 = children,
            SsrNodeType::Fragment(f) => f.borrow_mut().0 = children,
            _ => panic!("node type cannot have children"),
        };
    }

    fn remove_child(&self, child: &Self) {
        let mut children = match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => mem::take(&mut e.borrow_mut().children.0),
            SsrNodeType::Fragment(f) => mem::take(&mut f.borrow_mut().0),
            _ => panic!("node type cannot have children"),
        };

        let index = children
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == child).then(|| i))
            .expect("couldn't find child");
        children.remove(index);

        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => e.borrow_mut().children.0 = children,
            SsrNodeType::Fragment(f) => f.borrow_mut().0 = children,
            _ => panic!("node type cannot have children"),
        };
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        new.set_parent(Rc::downgrade(&self.0));

        let mut ele = self.unwrap_element().borrow_mut();
        let children = &mut ele.children.0;
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
        // Don't do anything. Events are attached on client side.
    }

    fn update_inner_text(&self, text: &str) {
        self.unwrap_text().borrow_mut().0 = text.to_string();
    }
}

impl fmt::Display for SsrNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(x) => write!(f, "{}", x.borrow()),
            SsrNodeType::Comment(x) => write!(f, "{}", x.borrow()),
            SsrNodeType::Text(x) => write!(f, "{}", x.borrow()),
            SsrNodeType::Fragment(x) => write!(f, "{}", x.borrow()),
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
pub struct Fragment(Vec<SsrNode>);

impl fmt::Display for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in &self.0 {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}

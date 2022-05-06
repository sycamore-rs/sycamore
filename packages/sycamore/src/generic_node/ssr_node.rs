//! Rendering backend for Server Side Rendering, aka. SSR.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::rc::{Rc, Weak};

use indexmap::map::IndexMap;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

use super::SycamoreElement;
use crate::generic_node::{GenericNode, Html};
use crate::reactive::*;
use crate::utils::hydrate::{get_next_id, with_hydration_context};
use crate::view::View;

static VOID_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr", "command", "keygen", "menuitem",
    ]
    .into_iter()
    .collect()
});

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

    /// Get an [`Element`], or `panic!` if wrong type.
    #[track_caller]
    pub fn unwrap_element(&self) -> &RefCell<Element> {
        match self.0.ty.as_ref() {
            SsrNodeType::Element(e) => e,
            _ => panic!("node is not an element"),
        }
    }

    /// Get a [`Text`], or `panic!` if wrong type.
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
    pub fn raw_text_node(html: &str) -> Self {
        SsrNode::new(SsrNodeType::RawText(RefCell::new(RawText(
            html.to_string(),
        ))))
    }
}

impl GenericNode for SsrNode {
    /// Although [`SsrNode`] is intended to be used on the server-side instead of in the browser,
    /// the event type is still [`web_sys::Event`] because it must support the same API as
    /// [`DomNode`](super::DomNode). Since event handlers will never be called on the server side
    /// anyways, it's okay to do this.
    type EventType = web_sys::Event;
    type PropertyType = JsValue;

    const USE_HYDRATION_CONTEXT: bool = true;

    fn element<T: SycamoreElement>() -> Self {
        let hk = get_next_id();
        let mut attributes = IndexMap::new();
        if let Some(hk) = hk {
            attributes.insert("data-hk".to_string(), format!("{}.{}", hk.0, hk.1));
        }
        Self::new(SsrNodeType::Element(RefCell::new(Element {
            name: Cow::Borrowed(T::TAG_NAME),
            attributes,
            children: Default::default(),
        })))
    }

    fn element_from_tag(tag: &str) -> Self {
        let hk = get_next_id();
        let mut attributes = IndexMap::new();
        if let Some(hk) = hk {
            attributes.insert("data-hk".to_string(), format!("{}.{}", hk.0, hk.1));
        }
        Self::new(SsrNodeType::Element(RefCell::new(Element {
            name: Cow::Owned(tag.to_string()),
            attributes,
            children: Default::default(),
        })))
    }

    fn text_node(text: &str) -> Self {
        Self::new(SsrNodeType::Text(RefCell::new(Text(text.to_string()))))
    }

    fn marker_with_text(text: &str) -> Self {
        Self::new(SsrNodeType::Comment(RefCell::new(Comment(
            text.to_string(),
        ))))
    }

    fn set_attribute(&self, name: &str, value: &str) {
        self.unwrap_element()
            .borrow_mut()
            .attributes
            .insert(name.to_string(), value.to_string());
    }

    fn remove_attribute(&self, name: &str) {
        self.unwrap_element().borrow_mut().attributes.remove(name);
    }

    fn set_class_name(&self, value: &str) {
        self.set_attribute("class", value);
    }

    fn add_class(&self, class: &str) {
        let attributes = &mut self.unwrap_element().borrow_mut().attributes;

        let classes = attributes.get_mut("class");

        if let Some(classes) = classes {
            // Make sure classes are unique.
            let mut class_set = HashSet::<_>::from_iter(classes.split(' '));

            class_set.insert(class);

            *classes = class_set.drain().collect::<Vec<_>>().join(" ");
        } else {
            attributes.insert("class".to_string(), class.to_owned());
        }
    }

    fn remove_class(&self, class: &str) {
        let attributes = &mut self.unwrap_element().borrow_mut().attributes;

        let classes = attributes.get_mut("class");

        if let Some(classes) = classes {
            // Make sure classes are unique.
            let mut class_set = HashSet::<_>::from_iter(classes.split(' '));

            class_set.remove(class);

            *classes = class_set.drain().collect::<Vec<_>>().join(" ");
        }
    }

    fn set_property(&self, _name: &str, _value: &JsValue) {
        // Noop.
    }

    fn remove_property(&self, _name: &str) {
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

    fn replace_child(&self, old: &Self, new: &Self) {
        new.set_parent(Rc::downgrade(&self.0));

        let mut ele = self.unwrap_element().borrow_mut();
        let children = &mut ele.children;
        let index = children
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c == old).then(|| i))
            .expect("the node to be replaced is not a child of this node");
        *children[index].0.parent.borrow_mut() = Weak::new();
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
                    .nth(1)
                    .cloned()
            }
            _ => panic!("node type cannot have children"),
        }
    }

    fn remove_self(&self) {
        self.parent_node()
            .expect("node must have a parent")
            .remove_child(self);
    }

    fn event<'a, F: FnMut(Self::EventType) + 'a>(&self, _cx: Scope<'a>, _name: &str, _handler: F) {
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

impl Html for SsrNode {
    const IS_BROWSER: bool = false;
}

trait WriteToString {
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
pub struct Element {
    name: Cow<'static, str>,
    attributes: IndexMap<String, String>,
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
pub struct Comment(String);

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
pub struct Text(String);

impl WriteToString for Text {
    fn write_to_string(&self, s: &mut String) {
        s.push_str(&html_escape::encode_text_minimal(&self.0));
    }
}

/// Un-escaped text node.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct RawText(String);

impl WriteToString for RawText {
    fn write_to_string(&self, s: &mut String) {
        s.push_str(&self.0);
    }
}

/// Render a [`View`] into a static [`String`]. Useful
/// for rendering to a string on the server side.
///
/// _This API requires the following crate features to be activated: `ssr`_
#[must_use]
pub fn render_to_string(view: impl FnOnce(Scope<'_>) -> View<SsrNode>) -> String {
    let mut ret = String::new();
    create_scope_immediate(|cx| {
        let v = with_hydration_context(|| view(cx));

        for node in v.flatten() {
            node.write_to_string(&mut ret);
        }
    });

    ret
}

/// Render a [`View`] into a static [`String`]. Useful
/// for rendering to a string on the server side.
///
/// Waits for suspense to be loaded before returning.
///
/// _This API requires the following crate features to be activated: `suspense`, `ssr`_
#[cfg(feature = "suspense")]
pub async fn render_to_string_await_suspense(
    view: impl FnOnce(Scope<'_>) -> View<SsrNode> + 'static,
) -> String {
    use futures::channel::oneshot;
    use sycamore_futures::spawn_local_scoped;

    use crate::utils::hydrate::with_hydration_context_async;

    let mut ret = String::new();
    let v = Rc::new(RefCell::new(None));
    let (sender, receiver) = oneshot::channel();
    let disposer = create_scope({
        let v = Rc::clone(&v);
        move |cx| {
            spawn_local_scoped(cx, async move {
                *v.borrow_mut() = Some(
                    with_hydration_context_async(async {
                        crate::suspense::await_suspense(cx, async { view(cx) }).await
                    })
                    .await,
                );
                sender
                    .send(())
                    .expect("receiving end should not be dropped");
            });
        }
    });
    receiver.await.expect("rendering should complete");
    let v = v.borrow().clone().unwrap();
    for node in v.flatten() {
        node.write_to_string(&mut ret);
    }

    // SAFETY: we are done with the scope now.
    unsafe {
        disposer.dispose();
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html;
    use crate::prelude::*;

    #[test]
    fn render_hello_world() {
        assert_eq!(
            render_to_string(|cx| view! { cx,
                "Hello World!"
            }),
            "Hello World!"
        );
    }

    #[test]
    fn render_escaped_text() {
        assert_eq!(
            render_to_string(|cx| view! { cx,
                "<script>Dangerous!</script>"
            }),
            "&lt;script>Dangerous!&lt;/script>"
        );
    }

    #[test]
    fn render_unescaped_html() {
        assert_eq!(
            render_to_string(|cx| view! { cx,
                div(dangerously_set_inner_html="<a>Html!</a>")
            }),
            "<div data-hk=\"0.0\"><a>Html!</a></div>"
        );
    }

    #[test]
    fn append_child() {
        let node = SsrNode::element::<html::div>();
        let p = SsrNode::element::<html::p>();
        let p2 = SsrNode::element::<html::p>();

        node.append_child(&p);
        node.append_child(&p2);

        // p and p2 parents should be updated
        assert_eq!(p.parent_node().as_ref(), Some(&node));
        assert_eq!(p2.parent_node().as_ref(), Some(&node));

        // node.first_child should be p
        assert_eq!(node.first_child().as_ref(), Some(&p));

        // p.next_sibling should be p2
        assert_eq!(p.next_sibling().as_ref(), Some(&p2));
    }

    #[test]
    fn remove_child() {
        let node = SsrNode::element::<html::div>();
        let p = SsrNode::element::<html::p>();

        node.append_child(&p);
        // p parent should be updated
        assert_eq!(p.parent_node().as_ref(), Some(&node));
        // node.first_child should be p
        assert_eq!(node.first_child().as_ref(), Some(&p));

        // remove p from node
        node.remove_child(&p);
        // p parent should be updated
        assert_eq!(p.parent_node().as_ref(), None);
        // node.first_child should be None
        assert_eq!(node.first_child().as_ref(), None);
    }

    #[test]
    fn remove_child_2() {
        let node = SsrNode::element::<html::div>();
        let p = SsrNode::element::<html::p>();
        let p2 = SsrNode::element::<html::p>();
        let p3 = SsrNode::element::<html::p>();

        node.append_child(&p);
        node.append_child(&p2);
        node.append_child(&p3);

        // node.first_child should be p
        assert_eq!(node.first_child().as_ref(), Some(&p));

        // remove p from node
        node.remove_child(&p);
        // p parent should be updated
        assert_eq!(p.parent_node().as_ref(), None);
        // node.first_child should be p2
        assert_eq!(node.first_child().as_ref(), Some(&p2));
    }
}

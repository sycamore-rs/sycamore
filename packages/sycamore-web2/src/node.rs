use crate::*;

/// A node in an HTML [`View`] tree.
pub struct HtmlNode {
    pub(crate) kind: HtmlNodeKind,
    pub(crate) node: Rc<OnceCell<web_sys::Node>>,
}

impl HtmlNode {
    pub(crate) fn element(el: HtmlElement, node: Rc<OnceCell<web_sys::Node>>) -> Self {
        HtmlNode {
            kind: HtmlNodeKind::Element(el),
            node,
        }
    }

    pub(crate) fn text(text: HtmlText, node: Rc<OnceCell<web_sys::Node>>) -> Self {
        HtmlNode {
            kind: HtmlNodeKind::Text(text),
            node,
        }
    }

    pub(crate) fn marker(node: Rc<OnceCell<web_sys::Node>>) -> Self {
        HtmlNode {
            kind: HtmlNodeKind::Marker,
            node,
        }
    }

    pub(crate) fn as_web_sys(&self) -> &Rc<OnceCell<web_sys::Node>> {
        &self.node
    }
}

pub(crate) enum HtmlNodeKind {
    Element(HtmlElement),
    Text(HtmlText),
    Marker,
}

impl From<&'static str> for View<HtmlNode> {
    fn from(t: &'static str) -> Self {
        View::node(HtmlNode::text(
            HtmlText { text: t.into() },
            Default::default(),
        ))
    }
}
impl From<String> for View<HtmlNode> {
    fn from(t: String) -> Self {
        View::node(HtmlNode::text(
            HtmlText { text: t.into() },
            Default::default(),
        ))
    }
}

impl<F: FnMut() -> U + 'static, U: Into<View<HtmlNode>> + Any + 'static> From<F>
    for View<HtmlNode>
{
    fn from(mut f: F) -> Self {
        // Specialize for U = String.
        if TypeId::of::<U>() == TypeId::of::<String>() {
            render_dynamic_text(move || {
                (&f() as &dyn Any).downcast_ref::<String>().unwrap().clone()
            })
        } else {
            render_dynamic_view(f)
        }
    }
}

fn render_dynamic_view<U: Into<View<HtmlNode>>>(
    mut f: impl FnMut() -> U + 'static,
) -> View<HtmlNode> {
    let mut nodes = vec![];

    let view = Rc::new(RefCell::new(None));
    let view2 = view.clone();
    let marker_node = Rc::new(OnceCell::new());

    let mut initial = true;
    create_effect(move || {
        if initial {
            // Create two nodes: one for the value itself and one as a marker.
            let marker = HtmlNode::marker(marker_node.clone());

            let value = f().into();
            nodes = value.as_web_sys();
            *view2.borrow_mut() = Some(View::from((value, View::node(marker))));

            initial = false;
        } else if is_client() {
            // First clear all the nodes.
            for node in &nodes {
                if let Some(node) = node.get() {
                    let parent = node.parent_node().unwrap();
                    parent.remove_child(&node).unwrap();
                }
            }

            // Then add the new nodes.
            nodes.clear();
            let view = f().into();
            nodes.extend(view.as_web_sys());

            // Finally, render all the new nodes.
            if let Some(marker) = marker_node.get() {
                let parent = marker.parent_node().unwrap();
                DomRenderer.render_before(&parent, view, Some(marker));
            }
        }
    });
    view.take().unwrap()
}

fn render_dynamic_text(mut f: impl FnMut() -> String + 'static) -> View<HtmlNode> {
    // Create an effect that will update the text content when the signal changes.
    let this = Rc::new(RefCell::new(None));
    let mut node = None;
    let mut initial = true;
    create_effect({
        let this = this.clone();
        move || {
            let text = f().into();
            if initial {
                node = Some(Default::default());
                *this.borrow_mut() = Some(HtmlNode::text(HtmlText { text }, node.clone().unwrap()));
                initial = false;
            } else if is_client() {
                if let Some(node) = node.as_ref().unwrap().get() {
                    node.set_text_content(Some(&text));
                }
            }
        }
    });

    View::node(this.take().unwrap())
}

impl From<HtmlNode> for View<HtmlNode> {
    fn from(node: HtmlNode) -> Self {
        View::node(node)
    }
}

impl View<HtmlNode> {
    /// Returns a list of web-sys nodes that are part of the view.
    pub fn as_web_sys(&self) -> Vec<Rc<OnceCell<web_sys::Node>>> {
        self.nodes
            .iter()
            .map(|node| node.as_web_sys().clone())
            .collect()
    }

    /// Ensures that there is always at least one node in the view. If the view is empty, a marker
    /// node is created.
    pub fn ensure_non_empty(self) -> Self {
        if self.nodes.is_empty() {
            View::node(HtmlNode::marker(Default::default()))
        } else {
            self
        }
    }
}

pub(crate) struct HtmlElement {
    pub tag: Cow<'static, str>,
    pub is_svg: bool,
    pub attributes: Vec<HtmlAttribute>,
    pub children: Vec<HtmlNode>,
    pub inner_html: Option<String>,
    pub events: Vec<(&'static str, Box<dyn FnMut(web_sys::Event)>)>,
}

pub(crate) struct HtmlText {
    pub text: Cow<'static, str>,
}

pub(crate) struct HtmlAttribute {
    pub name: Cow<'static, str>,
    pub value: Cow<'static, str>,
}

pub(crate) trait AsHtmlElement {
    #[allow(dead_code)]
    fn as_element(&self) -> &HtmlElement;
    fn as_mut_element(&mut self) -> &mut HtmlElement;
    fn to_node(&self) -> Rc<OnceCell<web_sys::Node>>;
}

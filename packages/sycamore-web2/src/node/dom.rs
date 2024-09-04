use wasm_bindgen::intern;

use crate::*;

/// View backend for rendering to the browser DOM.
pub struct DomNode {
    pub(crate) raw: web_sys::Node,
}

impl Renderer for Dom {
    fn append_child(&self, child: Self) {
        self.raw.append_child(&child.raw).unwrap();
    }

    fn append_dynamic(&self, mut dynamic: impl FnMut() -> crate::view::View<Self>) {
        let view = dynamic();
        for node in view.nodes {
            self.append_child(node);
        }
        // TODO: actually make this dynamic
    }
}

impl HtmlRenderer for Dom {
    fn create_element(tag: &str) -> Self {
        Self {
            raw: document().create_element(tag).unwrap().into(),
        }
    }

    fn create_element_ns(namespace: &str, tag: &str) -> Self {
        Self {
            raw: document()
                .create_element_ns(Some(namespace), tag)
                .unwrap()
                .into(),
        }
    }
}

///// Renderer for rendering a [`View`] to DOM nodes.
//#[derive(Debug, Clone)]
//pub(crate) struct DomRenderer;
//
//impl DomRenderer {
//    fn render_node(&self, node: HtmlNode, parent: &web_sys::Node, marker: Option<&web_sys::Node>)
// {        let raw_node = self.render_node_detatched(node);
//        if let Some(marker) = marker {
//            parent.insert_before(&raw_node, Some(marker)).unwrap();
//        } else {
//            parent.append_child(&raw_node).unwrap();
//        }
//    }
//
//    fn render_node_detatched(&self, node: HtmlNode) -> web_sys::Node {
//        let document = document();
//        let raw_node: web_sys::Node = match node.kind {
//            HtmlNodeKind::Element(node) => {
//                let tag = intern(node.tag.as_ref());
//                let el = if node.is_svg {
//                    let svg_ns = Some("http://www.w3.org/2000/svg");
//                    document.create_element_ns(svg_ns, tag).unwrap()
//                } else {
//                    document.create_element(tag).unwrap()
//                };
//                for attr in node.attributes {
//                    let name = intern(attr.name.as_ref());
//                    el.set_attribute(name, attr.value.as_ref()).unwrap();
//                }
//                for prop in node.props {
//                    let name = intern(prop.name.as_ref());
//                    js_sys::Reflect::set(&el, &JsValue::from(name), &prop.value).unwrap();
//                }
//                if let Some(inner_html) = node.inner_html {
//                    assert!(
//                        node.children.is_empty(),
//                        "inner_html and children are mutually exclusive"
//                    );
//                    el.set_inner_html(inner_html.as_ref());
//                } else {
//                    for child in node.children {
//                        self.render_node(child, &el, None);
//                    }
//                }
//                for (name, handler) in node.events {
//                    let closure =
//                        Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>).into_js_value();
//                    el.add_event_listener_with_callback(
//                        intern(name),
//                        closure.as_ref().unchecked_ref(),
//                    )
//                    .unwrap();
//                    // TODO: manage Closure in reactive scope
//                }
//                el.into()
//            }
//            HtmlNodeKind::Text(node) => document.create_text_node(node.text.as_ref()).into(),
//            HtmlNodeKind::Marker => document.create_comment("").into(),
//        };
//        node.node.set(raw_node.clone()).unwrap();
//        raw_node
//    }
//
//    pub fn render_before(&self, root: &web_sys::Node, view: View, marker: Option<&web_sys::Node>)
// {        for node in view.nodes {
//            self.render_node(node, root, marker);
//        }
//    }
//
//    pub fn render(&self, root: &web_sys::Node, view: View) {
//        self.render_before(root, view, None);
//    }
//
//    pub fn render_view_detatched(&self, view: View) {
//        for node in view.nodes {
//            self.render_node_detatched(node);
//        }
//    }
//}
//
///// Renderer for rendering a [`View`] by hydrating existing DOM nodes.
//#[derive(Default)]
//pub(crate) struct DomHydrateRenderer {
//    /// Keep track of current hydration state.
//    reg: HydrationRegistry,
//    /// ALl nodes with data-hk attribute, indexed by hydration key.
//    nodes: Vec<web_sys::Node>,
//}
//
//impl DomHydrateRenderer {
//    fn hydrate_node(&self, node: HtmlNode, parent: &web_sys::Node) {
//        let raw_node = match node.kind {
//            HtmlNodeKind::Element(node) => {
//                let key = self.reg.next_key();
//                let el = self.nodes[key as usize].clone();
//
//                // Attach event handlers.
//                for (name, handler) in node.events {
//                    let closure =
//                        Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>).into_js_value();
//                    el.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())
//                        .unwrap();
//                }
//
//                if node.inner_html.is_some() {
//                    assert!(
//                        node.children.is_empty(),
//                        "inner_html and children are mutually exclusive"
//                    );
//                    // We can't hydrate anything.
//                } else {
//                    // Hydrate the children.
//                    for child in node.children {
//                        self.hydrate_node(child, &el);
//                    }
//                }
//
//                el
//            }
//            HtmlNodeKind::Text(_) => {
//                // Iterate through parent's children until we find a blank marker node. Remove
// this                // marker node. The next node is the text node to hydrate.
//                let mut next = parent.first_child();
//                while let Some(current) = next {
//                    next = current.next_sibling();
//
//                    if current.node_type() == web_sys::Node::COMMENT_NODE
//                        && current.text_content().unwrap() == ""
//                    {
//                        parent.remove_child(&current).unwrap();
//                        break;
//                    }
//                }
//                if let Some(el) = next {
//                    debug_assert!(el.node_type() == web_sys::Node::TEXT_NODE);
//                    el
//                } else {
//                    panic!("no text node found after hydration marker")
//                }
//            }
//            HtmlNodeKind::Marker => {
//                // Iterate through parent's children until we find a marker node.
//                let mut next = parent.first_child();
//                let mut marker = None;
//                while let Some(current) = next {
//                    if current.node_type() == web_sys::Node::COMMENT_NODE
//                        && current.text_content().unwrap() == "/"
//                    {
//                        marker = Some(current);
//                        break;
//                    }
//                    next = current.next_sibling();
//                }
//                marker.expect("no marker node found with hydration key")
//            }
//        };
//        // Set the node to hydrate it.
//        node.node.set(raw_node).unwrap();
//    }
//
//    fn collect_nodes(&mut self, root: &web_sys::Node) {
//        // Query all nodes with data-hk attribute and store them inside self.nodes.
//        let query = root
//            .unchecked_ref::<web_sys::Element>()
//            .query_selector_all("[data-hk]")
//            .unwrap();
//
//        let n = query.length() as usize;
//        self.nodes.reserve(n);
//        for i in 0..n {
//            let node = query.get(i as u32).unwrap();
//            self.nodes.push(node);
//        }
//
//        // Now sort the nodes into the right order by hydration key.
//        for i in 0..n {
//            let key = self.nodes[i]
//                .unchecked_ref::<web_sys::Element>()
//                .get_attribute("data-hk")
//                .unwrap()
//                .parse::<usize>()
//                .unwrap();
//            self.nodes.swap(i, key);
//        }
//    }
//
//    pub fn render(&mut self, root: &web_sys::Node, view: View) {
//        self.collect_nodes(root);
//        for node in view.nodes {
//            self.hydrate_node(node, root);
//        }
//    }
//}
//
///// Render a [`View`] into the DOM.
///// Alias for [`render_to`] with `parent` being the `<body>` tag.
/////
///// _This API requires the following crate features to be activated: `dom`_
//pub fn render(view: impl FnOnce() -> View) {
//    render_to(view, &document().body().unwrap());
//}
//
///// Render a [`View`] under a `parent` node.
///// For rendering under the `<body>` tag, use [`render`] instead.
//pub fn render_to(view: impl FnOnce() -> View, parent: &web_sys::Node) {
//    // Do not call the destructor function, effectively leaking the scope.
//    let _ = create_root(|| render_in_scope(view, parent));
//}
//
///// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
/////
///// This function is intended to be used for injecting an ephemeral sycamore view into a
///// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
///// modal is closed).
/////
///// It is, however, preferable to have a single call to [`render`] or [`render_to`] at the top
///// level of your app long-term. For rendering a view that will never be unmounted from the dom,
///// use [`render_to`] instead. For rendering under the `<body>` tag, use [`render`] instead.
/////
///// It is expected that this function will be called inside a reactive root, usually created using
///// [`create_root`].
//pub fn render_in_scope(view: impl FnOnce() -> View, parent: &web_sys::Node) {
//    DomRenderer.render(parent, view());
//}
//
///// Render a [`View`] under a `parent` node by reusing existing nodes (client side
///// hydration).
/////
///// Alias for [`hydrate_to`] with `parent` being the `<body>` tag.
///// For rendering without hydration, use [`render`](super::render) instead.
//pub fn hydrate(view: impl FnOnce() -> View) {
//    hydrate_to(view, &document().body().unwrap());
//}
//
///// Render a [`View`] under a `parent` node by reusing existing nodes (client side
///// hydration).
/////
///// For rendering under the `<body>` tag, use [`hydrate`] instead.
///// For rendering without hydration, use [`render`](super::render) instead.
//pub fn hydrate_to(view: impl FnOnce() -> View, parent: &web_sys::Node) {
//    // Do not call the destructor function, effectively leaking the scope.
//    let _ = create_root(|| hydrate_in_scope(view, parent));
//}
//
///// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
/////
///// This function is intended to be used for injecting an ephemeral sycamore view into a
///// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
///// modal is closed).
/////
///// It is expected that this function will be called inside a reactive root, usually created using
///// [`create_root`].
//pub fn hydrate_in_scope(view: impl FnOnce() -> View, parent: &web_sys::Node) {
//    DomHydrateRenderer::default().render(parent, view());
//}

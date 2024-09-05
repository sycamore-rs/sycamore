use std::any::{Any, TypeId};

use wasm_bindgen::intern;

use crate::*;

/// View backend for rendering to the browser DOM.
pub struct DomNode {
    pub(crate) raw: web_sys::Node,
}

impl From<DomNode> for View<DomNode> {
    fn from(node: DomNode) -> Self {
        View::from_node(node)
    }
}

impl ViewNode for DomNode {
    fn append_child(&mut self, child: Self) {
        self.raw.append_child(&child.raw).unwrap();
    }

    fn create_dynamic_view<U: Into<View<Self>> + 'static>(
        mut f: impl FnMut() -> U + 'static,
    ) -> View<Self> {
        // If `view` is just a single text node, we can just return this node and set up an
        // effect to update its text value without ever creating more nodes.
        if TypeId::of::<U>() == TypeId::of::<String>() {
            create_effect_initial(move || {
                let view = f().into();
                debug_assert_eq!(
                    view.nodes.len(),
                    1,
                    "dynamic text view should have exactly one text node"
                );
                let node = view.nodes[0].as_web_sys().clone();
                (
                    Box::new(move || {
                        let text = f();
                        let text = (&text as &dyn Any).downcast_ref::<String>().unwrap();
                        node.set_text_content(Some(text));
                    }),
                    view,
                )
            })
        } else {
            let start = Self::create_marker_node();
            let view = f().into();
            // TODO: create effect
            let end = Self::create_marker_node();
            View::from((start, view, end))
        }
    }
}

impl ViewHtmlNode for DomNode {
    fn create_element(tag: Cow<'static, str>) -> Self {
        Self {
            raw: document().create_element(&tag).unwrap().into(),
        }
    }

    fn create_element_ns(namespace: &str, tag: Cow<'static, str>) -> Self {
        Self {
            raw: document()
                .create_element_ns(Some(namespace), &tag)
                .unwrap()
                .into(),
        }
    }

    fn create_text_node(text: Cow<'static, str>) -> Self {
        Self {
            raw: document().create_text_node(&text).into(),
        }
    }

    fn create_marker_node() -> Self {
        Self {
            raw: document().create_comment("").into(),
        }
    }

    fn set_attribute(&mut self, name: &'static str, value: MaybeDynString) {
        // FIXME: use setAttributeNS if SVG
        match value {
            MaybeDyn::Static(value) => {
                self.raw
                    .unchecked_ref::<web_sys::Element>()
                    .set_attribute(name, &value)
                    .unwrap();
            }
            MaybeDyn::Dynamic(mut f) => {
                let node = self.raw.clone().unchecked_into::<web_sys::Element>();
                create_effect(move || {
                    node.set_attribute(name, &f()).unwrap();
                });
            }
        }
    }

    fn set_bool_attribute(&mut self, name: &'static str, value: MaybeDynBool) {
        // FIXME: use setAttributeNS if SVG
        match value {
            MaybeDyn::Static(value) => {
                if value {
                    self.raw
                        .unchecked_ref::<web_sys::Element>()
                        .set_attribute(name, "")
                        .unwrap();
                }
            }
            MaybeDyn::Dynamic(mut f) => {
                let node = self.raw.clone().unchecked_into::<web_sys::Element>();
                create_effect(move || {
                    if f() {
                        node.set_attribute(name, "").unwrap();
                    } else {
                        node.remove_attribute(name).unwrap();
                    }
                });
            }
        }
    }

    fn set_property(&mut self, name: &'static str, value: MaybeDynJsValue) {
        match value {
            MaybeDyn::Static(value) => {
                assert!(js_sys::Reflect::set(&self.raw, &name.into(), &value).unwrap_throw())
            }
            MaybeDyn::Dynamic(mut f) => {
                let node = self.raw.clone().unchecked_into::<web_sys::Element>();
                create_effect(move || {
                    assert!(js_sys::Reflect::set(&node, &name.into(), &f()).unwrap_throw())
                });
            }
        }
    }

    fn set_event_handler(
        &mut self,
        name: &'static str,
        handler: impl FnMut(web_sys::Event) + 'static,
    ) {
        let cb = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        self.raw
            .add_event_listener_with_callback(name, cb.as_ref().unchecked_ref())
            .unwrap();
    }

    fn set_inner_html(&mut self, inner_html: Cow<'static, str>) {
        self.raw
            .unchecked_ref::<web_sys::Element>()
            .set_inner_html(&inner_html);
    }

    fn as_web_sys(&self) -> &web_sys::Node {
        &self.raw
    }
}

/// Render a [`View`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render(view: impl FnOnce() -> View) {
    render_to(view, &document().body().unwrap());
}

/// Render a [`View`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
pub fn render_to(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    // Do not call the destructor function, effectively leaking the scope.
    let _ = create_root(|| render_in_scope(view, parent));
}

/// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
///
/// This function is intended to be used for injecting an ephemeral sycamore view into a
/// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
/// modal is closed).
///
/// It is, however, preferable to have a single call to [`render`] or [`render_to`] at the top
/// level of your app long-term. For rendering a view that will never be unmounted from the dom,
/// use [`render_to`] instead. For rendering under the `<body>` tag, use [`render`] instead.
///
/// It is expected that this function will be called inside a reactive root, usually created using
/// [`create_root`].
pub fn render_in_scope(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    let nodes = view().nodes;
    for node in nodes {
        parent.append_child(node.as_web_sys()).unwrap();
    }
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration).
///
/// Alias for [`hydrate_to`] with `parent` being the `<body>` tag.
/// For rendering without hydration, use [`render`](super::render) instead.
pub fn hydrate(view: impl FnOnce() -> View) {
    hydrate_to(view, &document().body().unwrap());
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration).
///
/// For rendering under the `<body>` tag, use [`hydrate`] instead.
/// For rendering without hydration, use [`render`](super::render) instead.
pub fn hydrate_to(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    // Do not call the destructor function, effectively leaking the scope.
    let _ = create_root(|| hydrate_in_scope(view, parent));
}

/// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
///
/// This function is intended to be used for injecting an ephemeral sycamore view into a
/// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
/// modal is closed).
///
/// It is expected that this function will be called inside a reactive root, usually created using
/// [`create_root`].
pub fn hydrate_in_scope(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    todo!("hydrate")
}

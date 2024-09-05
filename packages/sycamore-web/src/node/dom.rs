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
            let start_node = start.as_web_sys().clone();
            let end = Self::create_marker_node();
            let end_node = end.as_web_sys().clone();
            let view = create_effect_initial(move || {
                let view = f().into();
                (
                    Box::new(move || {
                        let new = f().into();
                        if let Some(parent) = start_node.parent_node() {
                            // Clear all the old nodes away.
                            let old = iter::get_nodes_between(&start_node, &end_node);
                            for node in old {
                                parent.remove_child(&node).unwrap();
                            }
                            // Insert the new nodes in their place.
                            for node in new.nodes {
                                parent.insert_before(&node.raw, Some(&end_node)).unwrap();
                            }
                        }
                    }),
                    view,
                )
            });

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
        on_cleanup(|| drop(cb));
    }

    fn set_inner_html(&mut self, inner_html: Cow<'static, str>) {
        self.raw
            .unchecked_ref::<web_sys::Element>()
            .set_inner_html(&inner_html);
    }

    fn as_web_sys(&self) -> &web_sys::Node {
        &self.raw
    }

    fn from_web_sys(node: web_sys::Node) -> Self {
        Self { raw: node }
    }
}

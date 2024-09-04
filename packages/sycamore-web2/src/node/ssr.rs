use std::collections::HashSet;

use once_cell::sync::Lazy;

use crate::*;

pub enum SsrNode {
    Element {
        tag: Cow<'static, str>,
        attributes: Vec<(&'static str, Cow<'static, str>)>,
        bool_attributes: Vec<(&'static str, bool)>,
        children: Vec<Self>,
        inner_html: Option<Cow<'static, str>>,
        hk_key: u32,
    },
    Text {
        text: Cow<'static, str>,
    },
    Marker,
}

impl ViewNode for SsrNode {
    fn append_child(&self, _child: Self) {}

    fn append_dynamic(&self, _dynamic: impl FnMut() -> View<Self>) {}
}

impl ViewHtmlNode for SsrNode {
    fn create_element(tag: Cow<'static, str>) -> Self {
        let reg: HydrationRegistry = use_context();
        Self::Element {
            tag,
            attributes: Vec::new(),
            bool_attributes: Vec::new(),
            children: Vec::new(),
            inner_html: None,
            hk_key: reg.next_key(),
        }
    }

    fn create_element_ns(_namespace: &str, tag: Cow<'static, str>) -> Self {
        // SVG namespace is ignored in SSR mode.
        Self::create_element(tag)
    }

    fn create_text_node(text: Cow<'static, str>) -> Self {
        Self::Text { text }
    }

    fn create_marker_node() -> Self {
        Self::Marker
    }

    fn set_attribute(&mut self, name: &'static str, value: MaybeDynString) {
        match self {
            Self::Element { attributes, .. } => attributes.push((name, value.evaluate())),
            _ => panic!("can only set attribute on an element"),
        }
    }

    fn set_bool_attribute(&mut self, name: &'static str, value: MaybeDynBool) {
        match self {
            Self::Element {
                bool_attributes, ..
            } => bool_attributes.push((name, value.evaluate())),
            _ => panic!("can only set attribute on an element"),
        }
    }

    fn set_property(&mut self, _name: &'static str, _value: MaybeDynJsValue) {
        // Noop in SSR mode.
    }

    fn set_event_handler(
        &mut self,
        _name: &'static str,
        _handler: impl FnMut(web_sys::Event) + 'static,
    ) {
        // Noop in SSR mode.
    }

    fn set_inner_html(&mut self, inner_html: Cow<'static, str>) {
        match self {
            Self::Element {
                inner_html: slot, ..
            } => *slot = Some(inner_html),
            _ => panic!("can only set inner_html on an element"),
        }
    }

    fn as_web_sys(&self) -> &web_sys::Node {
        panic!("`as_web_sys()` is not supported in SSR mode")
    }
}

/// A list of all the void HTML elements. We need this to know how to render them to a string.
static VOID_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr", "command", "keygen", "menuitem",
    ]
    .into_iter()
    .collect()
});

fn render_recursive(node: SsrNode, buf: &mut String) {
    match node {
        SsrNode::Element {
            tag,
            attributes,
            bool_attributes,
            children,
            inner_html,
            hk_key,
        } => {
            buf.push('<');
            buf.push_str(&tag);
            for (name, value) in &attributes {
                buf.push(' ');
                buf.push_str(&name);
                buf.push_str("=\"");
                html_escape::encode_double_quoted_attribute_to_string(&value, buf);
                buf.push('"');
            }
            for (name, value) in &bool_attributes {
                if *value {
                    buf.push(' ');
                    buf.push_str(&name);
                }
            }

            buf.push_str(" data-hk=");
            buf.push_str(&hk_key.to_string());
            buf.push('>');

            let is_void = VOID_ELEMENTS.contains(tag.as_ref());

            if is_void {
                assert!(
                    children.is_empty() && inner_html.is_none(),
                    "void elements cannot have children or inner_html"
                );
                return;
            }
            if let Some(inner_html) = inner_html {
                assert!(
                    children.is_empty(),
                    "inner_html and children are mutually exclusive"
                );
                buf.push_str(&inner_html);
            } else {
                for child in children {
                    render_recursive(child, buf);
                }
            }

            if !is_void {
                buf.push_str("</");
                buf.push_str(&tag);
                buf.push('>');
            }
        }
        SsrNode::Text { text } => {
            buf.push_str("<!-->");
            html_escape::encode_text_to_string(&text, buf);
        }
        SsrNode::Marker => {
            buf.push_str("<!--/-->");
        }
    }
}

/// Render a [`View`] into a static [`String`]. Useful for rendering to a string on the server side.
#[must_use]
pub fn render_to_string(view: impl FnOnce() -> View) -> String {
    thread_local! {
        /// Use a static variable here so that we can reuse the same root for multiple calls to
        /// this function.
        static SSR_ROOT: Lazy<RootHandle> = Lazy::new(|| create_root(|| {}));
    }
    let mut buf = String::new();
    SSR_ROOT.with(|root| {
        root.dispose();
        root.run_in(|| {
            let handle = create_child_scope(|| {
                // We run this in a new scope so that we can dispose everything after we render it.
                provide_context(HydrationRegistry::new());

                let view = view();
                for node in view.nodes {
                    render_recursive(node, &mut buf);
                }
            });
            handle.dispose();
        });
    });
    buf
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::*;
    use crate::tags::*;

    fn check<T: Into<View>>(view: impl Fn() -> T, expect: Expect) {
        let actual = render_to_string(move || view().into());
        expect.assert_eq(&actual);
    }

    #[test]
    fn hello_world() {
        check(move || "Hello, world!", expect![[r#"<!-->Hello, world!"#]]);
    }

    #[test]
    fn render_escaped_text() {
        check(
            move || "<script>alert('xss')</script>",
            expect!["<!-->&lt;script&gt;alert('xss')&lt;/script&gt;"],
        );
    }

    #[test]
    fn render_inner_html() {
        check(
            move || div().dangerously_set_inner_html("<p>hello</p>"),
            expect!["<div data-hk=0><p>hello</p></div>"],
        );
    }

    #[test]
    fn render_void_element() {
        check(br, expect!["<br data-hk=0>"]);
        check(
            move || input().value("value"),
            expect![[r#"<input value="value" data-hk=0>"#]],
        );
    }

    #[test]
    fn fragments() {
        check(
            move || (p().children("1"), p().children("2"), p().children("3")),
            expect!["<p data-hk=0><!-->1</p><p data-hk=1><!-->2</p><p data-hk=2><!-->3</p>"],
        );
    }

    #[test]
    fn indexed() {
        check(
            move || {
                sycamore_macro::view! {
                    ul {
                        Indexed(
                            list=vec![1, 2],
                            view=|i| sycamore_macro::view! { li { (i) } },
                        )
                    }
                }
            },
            expect![
                "<ul data-hk=0><li data-hk=1><!-->1</li><li data-hk=2><!-->2</li><!--/--></ul>"
            ],
        );
    }

    #[test]
    fn bind() {
        // bind always attaches to a JS prop so it is not rendered in SSR.
        check(
            move || {
                let value = create_signal(String::new());
                sycamore_macro::view! {
                    input(bind:value=value)
                }
            },
            expect!["<input data-hk=0>"],
        );
    }

    #[test]
    fn svg_element() {
        check(
            move || {
                sycamore_macro::view! {
                    svg(xmlns="http://www.w2.org/2000/svg") {
                        rect()
                    }
                }
            },
            expect![[
                r#"<svg xmlns="http://www.w2.org/2000/svg" data-hk=0><rect data-hk=1></rect></svg>"#
            ]],
        );
        check(
            move || {
                sycamore_macro::view! {
                    svg_a()
                }
            },
            expect!["<a data-hk=0></a>"],
        );
    }
}

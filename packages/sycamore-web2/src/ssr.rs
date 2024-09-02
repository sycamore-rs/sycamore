use std::collections::HashSet;

use once_cell::sync::Lazy;

use crate::*;

/// A list of all the void HTML elements. We need this to know how to render them to a string.
static VOID_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr", "command", "keygen", "menuitem",
    ]
    .into_iter()
    .collect()
});

/// Renderer for server-side rendering (SSR).
#[derive(Default, Debug, Clone)]
pub(crate) struct SsrRenderer {
    reg: HydrationRegistry,
}

impl SsrRenderer {
    fn render_node(&self, buf: &mut String, node: HtmlNode) {
        match node.kind {
            HtmlNodeKind::Element(node) => {
                let key = self.reg.next_key();

                buf.push('<');
                buf.push_str(&node.tag);
                for attr in &node.attributes {
                    buf.push(' ');
                    buf.push_str(&attr.name);
                    buf.push_str("=\"");
                    html_escape::encode_double_quoted_attribute_to_string(&attr.value, buf);
                    buf.push('"');
                }

                buf.push_str(" data-hk=");
                buf.push_str(&key.to_string());
                buf.push('>');

                if VOID_ELEMENTS.contains(node.tag.as_ref()) {
                    assert!(
                        node.children.is_empty() && node.inner_html.is_none(),
                        "void elements cannot have children or inner_html"
                    );
                    return;
                }
                if let Some(inner_html) = node.inner_html {
                    assert!(
                        node.children.is_empty(),
                        "inner_html and children are mutually exclusive"
                    );
                    buf.push_str(&inner_html);
                } else {
                    for child in node.children {
                        self.render_node(buf, child);
                    }
                }

                buf.push_str("</");
                buf.push_str(&node.tag);
                buf.push('>');
            }
            HtmlNodeKind::Text(node) => {
                buf.push_str("<!-->");
                html_escape::encode_text_to_string(&node.text, buf);
            }
            HtmlNodeKind::Marker => {
                buf.push_str("<!--/-->");
            }
        }
    }

    pub fn render(&self, buf: &mut String, view: View) {
        for node in view.nodes {
            self.render_node(buf, node);
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

                // If the dom feature is also enabled, we use context to tell us at runtime that we
                // are in SSR mode.
                if cfg!(feature = "dom") {
                    provide_context(SsrMode);
                }
                SsrRenderer::default().render(&mut buf, view());
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
            expect![[r#"<svg xmlns="http://www.w2.org/2000/svg" data-hk=0><rect data-hk=1></rect></svg>"#]],
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

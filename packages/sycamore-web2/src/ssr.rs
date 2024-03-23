use std::collections::HashSet;

use once_cell::sync::Lazy;

use crate::*;

static VOID_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr", "command", "keygen", "menuitem",
    ]
    .into_iter()
    .collect()
});

#[derive(Default, Debug, Clone)]
pub(crate) struct SsrRenderer {
    reg: HydrationRegistry,
}

impl SsrRenderer {
    fn render_node(&self, buf: &mut String, node: HtmlNode) {
        match node.kind {
            HtmlNodeKind::Element(node) => {
                let key = self.reg.next_key();

                buf.push_str("<");
                buf.push_str(&node.tag);
                for attr in &node.attributes {
                    buf.push_str(" ");
                    buf.push_str(&attr.name);
                    buf.push_str("=\"");
                    html_escape::encode_double_quoted_attribute_to_string(&attr.value, buf);
                    buf.push_str("\"");
                }

                buf.push_str(" data-hk=");
                buf.push_str(&key.to_string());
                buf.push_str(">");

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
                buf.push_str(">");
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

    pub fn render(&self, buf: &mut String, view: View<HtmlNode>) {
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

                // If the dom feature is also enabled, we use context to tell us at runtime that we are in
                // SSR mode.
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
    use super::*;
    use expect_test::{expect, Expect};

    fn check(view: impl Into<View>, expected: Expect) {
        let actual = render_to_string(|| view.into());
        expected.assert_eq(&actual);
    }

    #[test]
    fn hello_world() {
        check("Hello, world!", expect![[r#"<!-->Hello, world!"#]]);
    }

    #[test]
    fn render_escaped_text() {
        check(
            "<script>alert('xss')</script>",
            expect!["<!-->&lt;script&gt;alert('xss')&lt;/script&gt;"],
        );
    }

    #[test]
    fn render_inner_html() {
        check(
            div().dangerously_set_inner_html("<p>hello</p>"),
            expect!["<div data-hk=0><p>hello</p></div>"],
        );
    }

    #[test]
    fn render_void_element() {
        check(br(), expect!["<br data-hk=0>"]);
    }
}

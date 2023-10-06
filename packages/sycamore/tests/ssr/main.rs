mod noderef;

use sycamore::prelude::*;

#[test]
fn hello_world() {
    let _ = create_root(|| {
        let node = view! {
            p { "Hello World!" }
        };
        assert_eq!(sycamore::render_to_string(|| node), "<p>Hello World!</p>");
    });
}

#[test]
fn reactive_text() {
    let _ = create_root(|| {
        let count = create_signal(0);
        let node = view! {
            p { (count.get()) }
        };
        assert_eq!(sycamore::render_to_string(|| node.clone()), "<p>0</p>");
        count.set(1);
        assert_eq!(sycamore::render_to_string(|| node.clone()), "<p>1</p>");
    });
}

#[test]
fn reactive_text_with_siblings() {
    let _ = create_root(|| {
        let count = create_signal(0);
        let node = view! {
            p { "before" (count.get()) "after" }
        };
        assert_eq!(
            sycamore::render_to_string(|| node.clone()),
            "<p>before<!--#-->0<!--/-->after</p>"
        );
        count.set(1);
        assert_eq!(
            sycamore::render_to_string(|| node.clone()),
            "<p>before<!--#-->1<!--/-->after</p>"
        );
    });
}

#[test]
fn self_closing_tag() {
    let _ = create_root(|| {
        let node = view! {
            div {
                input {}
                input(value="a")
            }
        };
        assert_eq!(
            sycamore::render_to_string(|| node),
            "<div><input/><input value=\"a\"/></div>"
        )
    });
}

#[test]
fn fragments() {
    let _ = create_root(|| {
        let node = view! {
            p { "1" }
            p { "2" }
            p { "3" }
        };
        assert_eq!(
            sycamore::render_to_string(|| node),
            "<p>1</p><p>2</p><p>3</p>"
        );
    });
}

#[test]
fn indexed() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);
        let node = view! {
            ul {
                Indexed(
                    iterable=*count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        let actual = sycamore::render_to_string(|| node.clone());
        assert_eq!(actual, "<ul><li>1</li><li>2</li></ul>");

        count.update(|count| count.push(3));
        let actual = sycamore::render_to_string(|| node.clone());
        assert_eq!(actual, "<ul><li>1</li><li>2</li><li>3</li></ul>");

        count.update(|count| count.remove(0));
        let actual = sycamore::render_to_string(|| node.clone());
        assert_eq!(actual, "<ul><li>2</li><li>3</li></ul>");
    });
}

#[test]
fn bind() {
    let _ = create_root(|| {
        let signal = create_signal(String::new());
        let node = view! {
            input(bind:value=signal)
        };
        let actual = sycamore::render_to_string(|| node);
        assert_eq!(actual, "<input/>");
    });
}

#[test]
#[ignore = "FIXME"]
fn using_cx_in_dyn_node_creates_nested_scope() {
    let _ = sycamore::render_to_string(|| {
        let outer_depth = use_scope_depth();
        let inner_depth = create_signal(0);
        let node = view! {
            p {
                ({
                    inner_depth.set(use_scope_depth());
                    view! { }
                })
            }
        };
        assert_eq!(inner_depth.get(), outer_depth + 1);
        node
    });
}

#[test]
fn ssr_no_hydrate_sub_tree() {
    let out = sycamore::render_to_string(|| {
        view! {
            div {
                p { "Hydrated" }
                sycamore::web::NoHydrate {
                    p { "But not this" }
                }
            }
        }
    });
    assert_eq!(
        out,
        r#"<div data-hk="0.0"><p>Hydrated</p><!--#--><div data-hk="1.0"><p>But not this</p></div><!--/--></div>"#
    );
}

#[test]
fn no_ssr_sub_tree_should_not_be_emitted_in_ssr() {
    let out = sycamore::render_to_string(|| {
        view! {
            div {
                p { "Rendered" }
                sycamore::web::NoSsr {
                    p { "But not this" }
                }
            }
        }
    });
    assert_eq!(
        out,
        r#"<div data-hk="0.0"><p>Rendered</p><!--#--><div data-hk="1.0"><!----></div><!--/--></div>"#
    );
}

mod svg {
    use super::*;

    #[test]
    fn ssr_svg_elements() {
        let out = sycamore::render_to_string(|| {
            view! {
                svg(xmlns="http://www.w3.org/2000/svg") {
                    rect(width=100, height=100, fill="red")
                }
            }
        });
        assert_eq!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" data-hk="0.0"><rect fill="red" width="100" height="100"></rect></svg>"#
        );
    }

    #[test]
    fn ssr_svg_elements_with_same_name_as_html_elements() {
        let out = sycamore::render_to_string(|| {
            view! {
                svg(xmlns="http://www.w3.org/2000/svg") {
                    svg_a {} // Should render as "<a></a>"
                }
            }
        });
        assert_eq!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" data-hk="0.0"><a></a></svg>"#
        );
    }
}

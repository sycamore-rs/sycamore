mod noderef;

use std::cell::Cell;

use sycamore::prelude::*;

#[test]
fn hello_world() {
    create_scope_immediate(|cx| {
        let node = view! { cx,
            p { "Hello World!" }
        };
        assert_eq!(sycamore::render_to_string(|_| node), "<p>Hello World!</p>");
    });
}

#[test]
fn reactive_text() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, 0);
        let node = view! { cx,
            p { (count.get()) }
        };
        assert_eq!(sycamore::render_to_string(|_| node.clone()), "<p>0</p>");
        count.set(1);
        assert_eq!(sycamore::render_to_string(|_| node.clone()), "<p>1</p>");
    });
}

#[test]
fn reactive_text_with_siblings() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, 0);
        let node = view! { cx,
            p { "before" (count.get()) "after" }
        };
        assert_eq!(
            sycamore::render_to_string(|_| node.clone()),
            "<p>before<!--#-->0<!--/-->after</p>"
        );
        count.set(1);
        assert_eq!(
            sycamore::render_to_string(|_| node.clone()),
            "<p>before<!--#-->1<!--/-->after</p>"
        );
    });
}

#[test]
fn self_closing_tag() {
    create_scope_immediate(|cx| {
        let node = view! { cx,
            div {
                input {}
                input(value="a")
            }
        };
        assert_eq!(
            sycamore::render_to_string(|_| node),
            "<div><input/><input value=\"a\"/></div>"
        )
    });
}

#[test]
fn fragments() {
    create_scope_immediate(|cx| {
        let node = view! { cx,
            p { "1" }
            p { "2" }
            p { "3" }
        };
        assert_eq!(
            sycamore::render_to_string(|_| node),
            "<p>1</p><p>2</p><p>3</p>"
        );
    });
}

#[test]
fn indexed() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);
        let node = view! { cx,
            ul {
                Indexed {
                    iterable: count,
                    view: |cx, item| view! { cx,
                        li { (item) }
                    },
                }
            }
        };

        let actual = sycamore::render_to_string(|_| node.clone());
        assert_eq!(actual, "<ul><li>1</li><li>2</li></ul>");

        count.set(count.get().iter().cloned().chain(Some(3)).collect());
        let actual = sycamore::render_to_string(|_| node.clone());
        assert_eq!(actual, "<ul><li>1</li><li>2</li><li>3</li></ul>");

        count.set(count.get()[1..].into());
        let actual = sycamore::render_to_string(|_| node.clone());
        assert_eq!(actual, "<ul><li>2</li><li>3</li></ul>");
    });
}

#[test]
fn bind() {
    create_scope_immediate(|cx| {
        let signal = create_signal(cx, String::new());
        let node = view! { cx,
            input(bind:value=signal)
        };
        let actual = sycamore::render_to_string(|_| node);
        assert_eq!(actual, "<input/>");
    });
}

#[test]
fn using_cx_in_dyn_node_creates_nested_scope() {
    let _ = sycamore::render_to_string(|cx| {
        let outer_depth = scope_depth(cx);
        let inner_depth = create_ref(cx, Cell::new(0));
        let node = view! { cx,
            p {
                ({
                    inner_depth.set(scope_depth(cx));
                    view! { cx, }
                })
            }
        };
        assert_eq!(inner_depth.get(), outer_depth + 1);
        node
    });
}

#[test]
fn ssr_no_hydrate_sub_tree() {
    let out = sycamore::render_to_string(|cx| {
        view! { cx,
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
        r#"<div data-hk="0.0"><p data-hk="0.1">Hydrated</p><!--#--><div data-hk="1.0"><p>But not this</p></div><!--/--></div>"#
    );
}

#[test]
fn no_ssr_sub_tree_should_not_be_emitted_in_ssr() {
    let out = sycamore::render_to_string(|cx| {
        view! { cx,
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
        r#"<div data-hk="0.0"><p data-hk="0.1">Rendered</p><!--#--><div data-hk="1.0"><!----></div><!--/--></div>"#
    );
}

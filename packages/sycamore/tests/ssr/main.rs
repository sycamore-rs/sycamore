use sycamore::prelude::*;

#[test]
fn hello_world() {
    create_scope_immediate(|ctx| {
        let node = view! { ctx,
            p { "Hello World!" }
        };
        assert_eq!(sycamore::render_to_string(|_| node), "<p>Hello World!</p>");
    });
}

#[test]
fn reactive_text() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(0);
        let node = view! { ctx,
            p { (count.get()) }
        };
        assert_eq!(sycamore::render_to_string(|_| node.clone()), "<p>0</p>");
        count.set(1);
        assert_eq!(sycamore::render_to_string(|_| node.clone()), "<p>1</p>");
    });
}

#[test]
fn reactive_text_with_siblings() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(0);
        let node = view! { ctx,
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
    create_scope_immediate(|ctx| {
        let node = view! { ctx,
            div {
                input
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
    create_scope_immediate(|ctx| {
        let node = view! { ctx,
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
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);
        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item| view! { ctx,
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
#[ignore]
fn bind() {
    // create_scope_immediate(|ctx| {
    //     let signal = ctx.create_signal(String::new());
    //     let node = view! { ctx,
    //         input(bind:value=signal)
    //     };
    //     let actual = sycamore::render_to_string(|_| node);
    //     assert_eq!(actual, "<input/>");
    // });
}

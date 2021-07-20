use sycamore::prelude::*;

#[test]
fn hello_world() {
    let _ = create_root(|| {
        let node = template! {
            p { "Hello World!" }
        };

        assert_eq!(sycamore::render_to_string(|| node), "<p>Hello World!</p>");
    });
}

#[test]
fn reactive_text() {
    let _ = create_root(|| {
        let (count, set_count) = create_signal(0);

        let node = template! {
            p { (count.get()) }
        };

        let node1 = node.clone();
        assert_eq!(sycamore::render_to_string(|| node1), "<p>0</p>");

        set_count.set(1);
        assert_eq!(sycamore::render_to_string(|| node), "<p>1</p>");
    });
}

#[test]
fn self_closing_tag() {
    let _ = create_root(|| {
        let node = template! {
            div {
                input
                input(value="a")
            }
        };

        assert_eq!(
            sycamore::render_to_string(|| node),
            "<div><input /><input value=\"a\" /></div>"
        );
    });
}

#[test]
fn fragments() {
    let _ = create_root(|| {
        let node = template! {
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

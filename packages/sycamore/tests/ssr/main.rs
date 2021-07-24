use sycamore::prelude::*;

#[test]
fn hello_world() {
    let node = template! {
        p { "Hello World!" }
    };

    assert_eq!(sycamore::render_to_string(|| node), "<p>Hello World!</p>");
}

#[test]
fn reactive_text() {
    let count = Signal::new(0);

    let node = cloned!((count) => template! {
        p { (count.get()) }
    });

    assert_eq!(
        sycamore::render_to_string(cloned!((node) => move || node)),
        "<p>0</p>"
    );

    count.set(1);
    assert_eq!(sycamore::render_to_string(|| node), "<p>1</p>");
}

#[test]
fn self_closing_tag() {
    let node = template! {
        div {
            input
            input(value="a")
        }
    };

    assert_eq!(
        sycamore::render_to_string(|| node),
        "<div><input /><input value=\"a\" /></div>"
    )
}

#[test]
fn fragments() {
    let node = template! {
        p { "1" }
        p { "2" }
        p { "3" }
    };

    assert_eq!(
        sycamore::render_to_string(|| node),
        "<p>1</p><p>2</p><p>3</p>"
    );
}

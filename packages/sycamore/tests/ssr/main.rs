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
fn reactive_text_with_siblings() {
    let count = Signal::new(0);

    let node = cloned!((count) => template! {
        p { "before" (count.get()) "after" }
    });

    assert_eq!(
        sycamore::render_to_string(cloned!((node) => move || node)),
        "<p>before0after</p>"
    );

    count.set(1);
    assert_eq!(sycamore::render_to_string(|| node), "<p>before1after</p>");
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
        "<div><input/><input value=\"a\"/></div>"
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

#[test]
fn indexed() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => template! {
        ul {
            Indexed(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    let actual = sycamore::render_to_string(|| node.clone());
    assert_eq!(actual, "<ul><li>1</li><li>2</li></ul>");

    count.set(count.get().iter().cloned().chain(Some(3)).collect());
    let actual = sycamore::render_to_string(|| node.clone());
    assert_eq!(actual, "<ul><li>1</li><li>2</li><li>3</li></ul>");

    count.set(count.get()[1..].into());
    let actual = sycamore::render_to_string(|| node.clone());
    assert_eq!(actual, "<ul><li>2</li><li>3</li></ul>");
}

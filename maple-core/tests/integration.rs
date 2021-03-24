use maple_core::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{Document, Node, Window};

wasm_bindgen_test_configure!(run_in_browser);

fn window() -> Window {
    web_sys::window().unwrap()
}

fn document() -> Document {
    window().document().unwrap()
}

/// Returns a [`Node`] referencing the test container with the contents cleared.
fn test_div() -> Node {
    if document()
        .query_selector("div#test-container")
        .unwrap()
        .is_none()
    {
        document()
            .body()
            .unwrap()
            .insert_adjacent_html("beforeend", r#"<div id="test-container"></div>"#)
            .unwrap();
    }

    let container = document()
        .query_selector("div#test-container")
        .unwrap()
        .unwrap();

    container.set_inner_html(""); // erase contents from previous test runs

    container.into()
}

#[wasm_bindgen_test]
fn hello_world() {
    let node = template! {
        p { "Hello World!" }
    };

    render_to(|| node, &test_div());

    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Hello World!"
    );
}

#[wasm_bindgen_test]
fn interpolation() {
    let text = "Hello Maple!";
    let node = template! {
        p { (text) }
    };

    render_to(|| node, &test_div());

    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Hello Maple!"
    );
}

#[wasm_bindgen_test]
fn reactive() {
    let count = Signal::new(0);

    let node = cloned!((count) => template! {
        p { (count.get()) }
    });

    render_to(|| node, &test_div());

    let p = document().query_selector("p").unwrap().unwrap();

    assert_eq!(p.text_content().unwrap(), "0");

    count.set(1);
    assert_eq!(p.text_content().unwrap(), "1");
}

#[wasm_bindgen_test]
fn non_keyed() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => template! {
        ul {
            Indexed(IndexedProps {
                iterable: count,
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_div());

    let p = document().query_selector("ul").unwrap().unwrap();

    assert_eq!(p.text_content().unwrap(), "12");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.push(3);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "123");

    count.set(count.get()[1..].into());
    assert_eq!(p.text_content().unwrap(), "23");
}

#[wasm_bindgen_test]
fn keyed() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => template! {
        ul {
            Keyed(KeyedProps {
                iterable: count,
                template: |item| template! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    render_to(|| node, &test_div());

    let p = document().query_selector("ul").unwrap().unwrap();

    assert_eq!(p.text_content().unwrap(), "12");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.push(3);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "123");

    count.set(count.get()[1..].into());
    assert_eq!(p.text_content().unwrap(), "23");
}

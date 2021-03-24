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

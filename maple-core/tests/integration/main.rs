pub mod keyed;
pub mod non_keyed;

use maple_core::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{Document, HtmlElement, Node, Window};

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
        &document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .outer_html(),
        "<p>Hello World!</p>"
    );
}

#[wasm_bindgen_test]
fn hello_world_noderef() {
    let p_ref = NodeRef::new();

    let node = template! {
        p(ref=p_ref) { "Hello World!" }
    };

    render_to(|| node, &test_div());

    assert_eq!(
        &p_ref
            .get::<DomNode>()
            .unchecked_into::<HtmlElement>()
            .outer_html(),
        "<p>Hello World!</p>"
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
fn reactive_text() {
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
fn reactive_attribute() {
    let count = Signal::new(0);

    let node = cloned!((count) => template! {
        span(attribute=count.get())
    });

    render_to(|| node, &test_div());

    let span = document().query_selector("span").unwrap().unwrap();

    assert_eq!(span.get_attribute("attribute").unwrap(), "0");

    count.set(1);
    assert_eq!(span.get_attribute("attribute").unwrap(), "1");
}

#[wasm_bindgen_test]
fn noderefs() {
    let noderef = NodeRef::new();

    let node = template! {
        div {
            input(ref=noderef)
        }
    };

    render_to(|| node, &test_div());

    let input_ref = document().query_selector("input").unwrap().unwrap();

    assert_eq!(
        Node::from(input_ref),
        noderef.get::<DomNode>().unchecked_into()
    );
}

#[wasm_bindgen_test]
fn fragments() {
    let node = template! {
        p { "1" }
        p { "2" }
        p { "3" }
    };

    render_to(|| node, &test_div());

    let test_container = document().query_selector("#test-container").unwrap().unwrap();

    assert_eq!(
        test_container.text_content().unwrap(),
        "123"
    );
}

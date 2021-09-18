pub mod cleanup;
pub mod keyed;
pub mod non_keyed;
pub mod portal;
pub mod reconcile;
pub mod render;

use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Event, HtmlElement, HtmlInputElement, Node, Window};

wasm_bindgen_test_configure!(run_in_browser);

fn window() -> Window {
    web_sys::window().unwrap()
}

fn document() -> Document {
    window().document().unwrap()
}

/// Returns a [`Node`] referencing the test container with the contents cleared.
fn test_container() -> Node {
    if document()
        .query_selector("test-container#test-container")
        .unwrap()
        .is_none()
    {
        document()
            .body()
            .unwrap()
            .insert_adjacent_html(
                "beforeend",
                r#"<test-container id="test-container"></test-container>"#,
            )
            .unwrap();
    }

    let container = document()
        .query_selector("test-container#test-container")
        .unwrap()
        .unwrap();

    container.set_inner_html(""); // erase contents from previous test runs

    container.into()
}

#[wasm_bindgen_test]
fn empty_template() {
    let node = template! {};

    sycamore::render_to(|| node, &test_container());

    assert_eq!(
        document()
            .query_selector("#test-container")
            .unwrap()
            .unwrap()
            .inner_html(),
        "<!---->"
    );
}

#[wasm_bindgen_test]
fn hello_world() {
    let node = template! {
        p { "Hello World!" }
    };

    sycamore::render_to(|| node, &test_container());

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

    sycamore::render_to(|| node, &test_container());

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
    let text = "Hello Sycamore!";
    let node = template! {
        p { (text) }
    };

    sycamore::render_to(|| node, &test_container());

    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Hello Sycamore!"
    );
}

#[wasm_bindgen_test]
fn template_interpolation() {
    let text = template! { "Hello Sycamore!" };
    let node = template! {
        p {
            (text)
        }
    };

    sycamore::render_to(|| node, &test_container());

    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Hello Sycamore!"
    );
}

#[wasm_bindgen_test]
fn template_interpolation_if_else() {
    let show = Signal::new(true);
    let node = cloned!((show) => template! {
        p {
            (if *show.get() {
                template! { "Hello Sycamore!" }
            } else {
                template! {}
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Hello Sycamore!"
    );

    show.set(false);
    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        ""
    );

    show.set(true);
    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Hello Sycamore!"
    );
}

#[wasm_bindgen_test]
fn template_interpolation_if_else_with_sibling() {
    let show = Signal::new(true);
    let node = cloned!((show) => template! {
        div { "Before" }
        (if *show.get() {
            template! { p { "Hello Sycamore!" } }
        } else {
            template! { p { "" }}
        })
    });

    sycamore::render_to(|| node, &test_container());

    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Hello Sycamore!"
    );

    show.set(false);
    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        ""
    );

    show.set(true);
    assert_eq!(
        document()
            .query_selector("p")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Hello Sycamore!"
    );
}

#[wasm_bindgen_test]
fn template_interpolation_nested_reactivity() {
    let count = Signal::new(0);
    let text = cloned!((count) => template! { p { (count.get() ) } });
    let node = template! {
        p {
            (text)
        }
    };

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("p").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "0");

    count.set(1);
    assert_eq!(p.text_content().unwrap(), "1");
}

#[wasm_bindgen_test]
fn reactive_text() {
    let count = Signal::new(0);

    let node = cloned!((count) => template! {
        p { (count.get()) }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("p").unwrap().unwrap();

    assert_eq!(p.text_content().unwrap(), "0");

    count.set(1);
    assert_eq!(p.text_content().unwrap(), "1");
}

#[wasm_bindgen_test]
fn reactive_text_do_not_destroy_previous_children() {
    let count = Signal::new(0);

    let node = cloned!((count) => template! {
        p { "Value: " (count.get()) }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("p").unwrap().unwrap();

    assert_eq!(p.text_content().unwrap(), "Value: 0");

    count.set(1);
    assert_eq!(p.text_content().unwrap(), "Value: 1");
}

#[wasm_bindgen_test]
fn reactive_attribute() {
    let count = Signal::new(0);

    let node = cloned!((count) => template! {
        span(attribute=count.get())
    });

    sycamore::render_to(|| node, &test_container());

    let span = document().query_selector("span").unwrap().unwrap();

    assert_eq!(span.get_attribute("attribute").unwrap(), "0");

    count.set(1);
    assert_eq!(span.get_attribute("attribute").unwrap(), "1");
}

#[wasm_bindgen_test]
fn two_way_bind_to_props() {
    let value = Signal::new(String::new());
    let value2 = value.clone();

    sycamore::render_to(
        || {
            cloned!((value) => template! {
                input(bind:value=value)
                p { (value2.get()) }
            })
        },
        &test_container(),
    );

    let input = document()
        .query_selector("input")
        .unwrap()
        .unwrap()
        .unchecked_into::<HtmlInputElement>();

    value.set("abc".to_string());
    assert_eq!(
        js_sys::Reflect::get(&input, &"value".into()).unwrap(),
        "abc"
    );

    js_sys::Reflect::set(&input, &"value".into(), &"def".into()).unwrap();
    input.dispatch_event(&Event::new("input").unwrap()).unwrap();
    assert_eq!(value.get().as_str(), "def");
}

#[wasm_bindgen_test]
fn noderefs() {
    let noderef = NodeRef::new();

    let node = template! {
        div {
            input(ref=noderef)
        }
    };

    sycamore::render_to(|| node, &test_container());

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

    sycamore::render_to(|| node, &test_container());

    let test_container = document()
        .query_selector("#test-container")
        .unwrap()
        .unwrap();

    assert_eq!(test_container.text_content().unwrap(), "123");
}

#[wasm_bindgen_test]
fn fragments_text_nodes() {
    let node = template! {
        "1"
        "2"
        "3"
    };

    sycamore::render_to(|| node, &test_container());

    let test_container = document()
        .query_selector("#test-container")
        .unwrap()
        .unwrap();

    assert_eq!(test_container.text_content().unwrap(), "123");
}

#[wasm_bindgen_test]
fn dyn_fragment_reuse_nodes() {
    let nodes = vec![template! { "1" }, template! { "2" }, template! { "3" }];

    sycamore::render_to(
        cloned!((nodes) =>
            move || Template::new_dyn(move || Template::new_fragment(nodes.clone()))
        ),
        &test_container(),
    );

    let p = document()
        .query_selector("#test-container")
        .unwrap()
        .unwrap();

    assert_eq!(p.text_content().unwrap(), "123");
    assert!(p.first_child() == nodes[0].as_node().map(|node| node.inner_element()));
}

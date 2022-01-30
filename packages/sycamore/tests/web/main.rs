#[cfg(all(
    feature = "experimental-hydrate",
    feature = "experimental-builder-html"
))]
pub mod builder_hydrate;
pub mod cleanup;
#[cfg(feature = "experimental-hydrate")]
pub mod hydrate;
pub mod indexed;
pub mod keyed;
pub mod portal;
pub mod reconcile;
pub mod render;

use sycamore::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement, Node, Window};

wasm_bindgen_test_configure!(run_in_browser);

fn window() -> Window {
    web_sys::window().unwrap()
}

fn document() -> Document {
    window().document().unwrap()
}

/// Returns a [`Element`] referencing the test container with the contents cleared.
fn test_container() -> Element {
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

    container
}

#[wasm_bindgen_test]
fn empty_template() {
    sycamore::render_to(|_| View::empty(), &test_container());
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
    sycamore::render_to(
        |ctx| {
            view! { ctx,
                p { "Hello World!" }
            }
        },
        &test_container(),
    );
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

    sycamore::render_to(
        |ctx| {
            view! { ctx,
                p(ref=p_ref) { "Hello World!" }
            }
        },
        &test_container(),
    );

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
    sycamore::render_to(
        |ctx| {
            view! { ctx,
                p { (text) }
            }
        },
        &test_container(),
    );

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
    sycamore::render_to(
        |ctx| {
            let text = view! { ctx, "Hello Sycamore!" };
            view! { ctx,
                p {
                    (text)
                }
            }
        },
        &test_container(),
    );
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
    create_scope_immediate(|ctx| {
        let show = ctx.create_signal(true);
        let node = view! { ctx,
            p {
                (if *show.get() {
                    view! { ctx, "Hello Sycamore!" }
                } else {
                    view! { ctx, }
                })
            }
        };
        sycamore::render_to(|_| node, &test_container());
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
    });
}

#[wasm_bindgen_test]
fn template_interpolation_if_else_with_sibling() {
    create_scope_immediate(|ctx| {
        let show = ctx.create_signal(true);
        let node = view! { ctx,
            div { "Before" }
            (if *show.get() {
                view! { ctx, p { "Hello Sycamore!" } }
            } else {
                view! { ctx, p { "" }}
            })
        };
        sycamore::render_to(|_| node, &test_container());
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
    });
}

#[wasm_bindgen_test]
fn template_interpolation_nested_reactivity() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(0);
        let text = view! { ctx, p { (count.get() ) } };
        let node = view! { ctx,
            p {
                (text)
            }
        };

        sycamore::render_to(|_| node, &test_container());
        let p = document().query_selector("p").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "0");

        count.set(1);
        assert_eq!(p.text_content().unwrap(), "1");
    });
}

#[wasm_bindgen_test]
fn reactive_text() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(0);

        let node = view! { ctx,
            p { (count.get()) }
        };

        sycamore::render_to(|_| node, &test_container());
        let p = document().query_selector("p").unwrap().unwrap();

        assert_eq!(p.text_content().unwrap(), "0");

        count.set(1);
        assert_eq!(p.text_content().unwrap(), "1");
    });
}

#[wasm_bindgen_test]
fn reactive_text_do_not_destroy_previous_children() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(0);

        let node = view! { ctx,
            p { "Value: " (count.get()) }
        };

        sycamore::render_to(|_| node, &test_container());
        let p = document().query_selector("p").unwrap().unwrap();

        assert_eq!(p.text_content().unwrap(), "Value: 0");

        count.set(1);
        assert_eq!(p.text_content().unwrap(), "Value: 1");
    });
}

#[wasm_bindgen_test]
fn reactive_attribute() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(0);

        let node = view! { ctx,
            span(attribute=count.get())
        };

        sycamore::render_to(|_| node, &test_container());
        let span = document().query_selector("span").unwrap().unwrap();

        assert_eq!(span.get_attribute("attribute").unwrap(), "0");

        count.set(1);
        assert_eq!(span.get_attribute("attribute").unwrap(), "1");
    });
}

#[wasm_bindgen_test]
#[ignore]
fn two_way_bind_to_props() {
    // create_scope_immediate(|ctx| {
    //     let value = ctx.create_signal(String::new());
    //     let value2 = value.clone();

    //     sycamore::render_to(
    //         || {
    //             cloned!((value) => view! {
    //                 input(bind:value=value)
    //                 p { (value2.get()) }
    //             })
    //         },
    //         &test_container(),
    //     );

    //     let input = document()
    //         .query_selector("input")
    //         .unwrap()
    //         .unwrap()
    //         .unchecked_into::<HtmlInputElement>();

    //     value.set("abc".to_string());
    //     assert_eq!(
    //         js_sys::Reflect::get(&input, &"value".into()).unwrap(),
    //         "abc"
    //     );

    //     js_sys::Reflect::set(&input, &"value".into(), &"def".into()).unwrap();
    //     input.dispatch_event(&Event::new("input").unwrap()).unwrap();
    //     assert_eq!(value.get().as_str(), "def");
    // });
}

#[wasm_bindgen_test]
fn noderefs() {
    create_scope_immediate(|ctx| {
        let noderef = ctx.create_node_ref();
        let node = view! { ctx,
            div {
                input(ref=noderef)
            }
        };

        sycamore::render_to(|_| node, &test_container());
        let input_ref = document().query_selector("input").unwrap().unwrap();

        assert_eq!(
            Node::from(input_ref),
            noderef.get::<DomNode>().unchecked_into()
        );
    });
}

#[wasm_bindgen_test]
fn fragments() {
    create_scope_immediate(|ctx| {
        let node = view! { ctx,
            p { "1" }
            p { "2" }
            p { "3" }
        };
        sycamore::render_to(|_| node, &test_container());
        let test_container = document()
            .query_selector("#test-container")
            .unwrap()
            .unwrap();

        assert_eq!(test_container.text_content().unwrap(), "123");
    });
}

#[wasm_bindgen_test]
fn fragments_text_nodes() {
    create_scope_immediate(|ctx| {
        let node = view! { ctx,
            "1"
            "2"
            "3"
        };

        sycamore::render_to(|_| node, &test_container());
        let test_container = document()
            .query_selector("#test-container")
            .unwrap()
            .unwrap();

        assert_eq!(test_container.text_content().unwrap(), "123");
    });
}

#[wasm_bindgen_test]
fn dyn_fragment_reuse_nodes() {
    create_scope_immediate(|ctx| {
        let nodes = vec![view! { ctx, "1" }, view! { ctx, "2" }, view! { ctx, "3" }];

        sycamore::render_to(
            {
                let nodes = nodes.clone();
                |_| View::new_dyn(ctx, move || View::new_fragment(nodes.clone()))
            },
            &test_container(),
        );

        let p = document()
            .query_selector("#test-container")
            .unwrap()
            .unwrap();

        assert_eq!(p.text_content().unwrap(), "123");
        assert!(p.first_child() == nodes[0].as_node().map(|node| node.inner_element()));
    });
}

#[cfg(all(feature = "hydrate"))]
pub mod builder_hydrate;
pub mod cleanup;
#[cfg(feature = "hydrate")]
pub mod hydrate;
pub mod indexed;
pub mod keyed;
pub mod portal;
pub mod reconcile;
pub mod render;
pub mod svg;

mod utils;

use sycamore::generic_node::GenericNodeElements;
use sycamore::prelude::*;
use sycamore::web::html;
use utils::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Element, Event, HtmlElement, HtmlInputElement};

wasm_bindgen_test_configure!(run_in_browser);

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

    let container = query("test-container#test-container");

    container.set_inner_html(""); // erase contents from previous test runs

    container
}

#[wasm_bindgen_test]
fn empty_template() {
    sycamore::render_to(|| View::empty(), &test_container());
    assert_eq!(query("#test-container").inner_html(), "<!---->");
}

#[wasm_bindgen_test]
fn hello_world() {
    sycamore::render_to(
        || {
            view! {
                p { "Hello World!" }
            }
        },
        &test_container(),
    );
    assert_eq!(query("p").outer_html(), "<p>Hello World!</p>");
}

#[wasm_bindgen_test]
fn hello_world_noderef() {
    let _ = create_root(|| {
        let p_ref = NodeRef::new();

        sycamore::render_in_scope(
            move || {
                view! {
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
    });
}

#[wasm_bindgen_test]
fn interpolation() {
    let text = "Hello Sycamore!";
    sycamore::render_to(
        move || {
            view! {
                p { (text) }
            }
        },
        &test_container(),
    );

    assert_text_content!(query("p"), "Hello Sycamore!");
}

#[wasm_bindgen_test]
fn template_interpolation() {
    sycamore::render_to(
        || {
            let text = view! { "Hello Sycamore!" };
            view! {
                p {
                    (text)
                }
            }
        },
        &test_container(),
    );
    assert_text_content!(query("p"), "Hello Sycamore!");
}

#[wasm_bindgen_test]
fn template_interpolation_if_else() {
    let _ = create_root(|| {
        let show = create_signal(true);
        let node = view! {
            p {
                (if show.get() {
                    view! { "Hello Sycamore!" }
                } else {
                    view! { "Hidden" }
                })
            }
        };
        sycamore::render_in_scope(|| node, &test_container());
        assert_text_content!(query("p"), "Hello Sycamore!");

        show.set(false);
        assert_text_content!(query("p"), "Hidden");

        show.set(true);
        assert_text_content!(query("p"), "Hello Sycamore!");
    });
}

#[wasm_bindgen_test]
fn template_interpolation_if_else_with_sibling() {
    let _ = create_root(|| {
        let show = create_signal(true);
        let node = view! {
            div { "Before" }
            (if show.get() {
                view! { p { "Hello Sycamore!" } }
            } else {
                view! { p { "Hidden" }}
            })
        };
        sycamore::render_in_scope(|| node, &test_container());
        assert_text_content!(query("p"), "Hello Sycamore!");

        show.set(false);
        assert_text_content!(query("p"), "Hidden");

        show.set(true);
        assert_text_content!(query("p"), "Hello Sycamore!");
    });
}

#[wasm_bindgen_test]
fn template_interpolation_nested_reactivity() {
    let _ = create_root(|| {
        let count = create_signal(0);
        let text = view! { p { (count.get() ) } };
        let node = view! {
            p {
                (text)
            }
        };

        sycamore::render_in_scope(|| node, &test_container());
        let p = query("p");
        assert_text_content!(p, "0");

        count.set(1);
        assert_text_content!(p, "1");
    });
}

#[wasm_bindgen_test]
fn reactive_text() {
    let _ = create_root(|| {
        let count = create_signal(0);

        let node = view! {
            p { (count.get()) }
        };

        sycamore::render_in_scope(|| node, &test_container());
        let p = query("p");

        assert_text_content!(p, "0");

        count.set(1);
        assert_text_content!(p, "1");
    });
}

#[wasm_bindgen_test]
fn reactive_text_do_not_destroy_previous_children() {
    let _ = create_root(|| {
        let count = create_signal(0);

        let node = view! {
            p { "Value: " (count.get()) }
        };

        sycamore::render_in_scope(|| node, &test_container());
        let p = query("p");

        assert_text_content!(p, "Value: 0");

        count.set(1);
        assert_text_content!(p, "Value: 1");
    });
}

#[wasm_bindgen_test]
fn reactive_attribute() {
    let _ = create_root(|| {
        let count = create_signal(0);

        let node = view! {
            span(attribute=count.get())
        };

        sycamore::render_in_scope(|| node, &test_container());
        let span = query("span");

        assert_eq!(span.get_attribute("attribute").unwrap(), "0");

        count.set(1);
        assert_eq!(span.get_attribute("attribute").unwrap(), "1");
    });
}

#[wasm_bindgen_test]
fn reactive_property() {
    let _ = create_root(|| {
        let indeterminate = create_signal(true);

        let node = view! {
            input(type="checkbox", prop:indeterminate=indeterminate.get())
        };

        sycamore::render_in_scope(|| node, &test_container());
        let input: HtmlInputElement = query_into("input");

        assert!(input.indeterminate());

        indeterminate.set(false);
        assert!(!input.indeterminate());
    });
}

#[wasm_bindgen_test]
fn static_property() {
    let _ = create_root(|| {
        let node = view! {
            input(prop:checked=true)
        };

        sycamore::render_in_scope(|| node, &test_container());
        let input: HtmlInputElement = query_into("input");

        assert!(input.checked());
    });
}

#[wasm_bindgen_test]
#[ignore]
fn two_way_bind_to_props() {
    let _ = create_root(|| {
        let value = create_signal(String::new());

        let node = view! {
            input(bind:value=value)
            p { (value.get_clone()) }
        };

        sycamore::render_in_scope(|| node, &test_container());
        let input: HtmlInputElement = query_into("input");

        value.set("abc".to_string());
        assert_eq!(input.value(), "abc");

        input.set_value("def");
        input.dispatch_event(&Event::new("input").unwrap()).unwrap();
        assert_eq!(value.get_clone().as_str(), "def");
    });
}

#[wasm_bindgen_test]
fn two_way_bind_to_value_as_number() {
    let _ = create_root(|| {
        let value = create_signal(1.0);

        let node = view! {
            input(type="range", bind:valueAsNumber=value) // Note that type must be "range" or "number"
        };

        sycamore::render_in_scope(|| node, &test_container());
        let input: HtmlInputElement = query_into("input");
        assert_eq!(input.value_as_number(), 1.0);

        value.set(2.0);
        assert_eq!(input.value_as_number(), 2.0);

        input.set_value_as_number(3.0);
        input.dispatch_event(&Event::new("input").unwrap()).unwrap();
        assert_eq!(value.get(), 3.0);
    });
}

#[wasm_bindgen_test]
fn noderefs() {
    let _ = create_root(|| {
        let noderef = create_node_ref();
        let node = view! {
            div {
                input(ref=noderef)
            }
        };

        sycamore::render_in_scope(|| node, &test_container());
        let input_ref = query("input");

        assert_eq!(input_ref, noderef.get::<DomNode>().unchecked_into());
    });
}

#[wasm_bindgen_test]
fn noderef_reactivity_test() {
    let _ = create_root(|| {
        let counter = create_signal(0);
        let node_ref: NodeRef<DomNode> = create_node_ref();

        let _ = view! {
            div(ref=node_ref)
        };

        create_effect(move || {
            node_ref.get::<DomNode>();
            counter.set(counter.get() + 1);
        });
        assert_eq!(counter.get(), 1);

        let _ = view! {
            div(ref=node_ref)
        };

        assert_eq!(counter.get(), 2);
    });
}

#[wasm_bindgen_test]
fn fragments() {
    let _ = create_root(|| {
        let node = view! {
            p { "1" }
            p { "2" }
            p { "3" }
        };
        sycamore::render_in_scope(|| node, &test_container());

        assert_text_content!(query("#test-container"), "123");
    });
}

#[wasm_bindgen_test]
fn fragments_text_nodes() {
    let _ = create_root(|| {
        let node = view! {
            "1"
            "2"
            "3"
        };

        sycamore::render_in_scope(|| node, &test_container());

        assert_text_content!(query("#test-container"), "123");
    });
}

#[wasm_bindgen_test]
fn dyn_fragment_reuse_nodes() {
    let _ = create_root(|| {
        let nodes = vec![view! { "1" }, view! { "2" }, view! { "3" }];

        sycamore::render_in_scope(
            {
                let nodes = nodes.clone();
                || View::new_dyn(move || View::new_fragment(nodes.clone()))
            },
            &test_container(),
        );

        let p = query("#test-container");

        assert_text_content!(p, "123");
        assert!(p.first_child() == nodes[0].as_node().map(|node| node.to_web_sys()));
    });
}

#[wasm_bindgen_test]
fn dom_node_add_class_splits_at_whitespace() {
    let node = DomNode::element::<html::div>();
    node.add_class("my_class");
    assert_eq!(
        node.to_web_sys().unchecked_into::<Element>().class_name(),
        "my_class"
    );
    node.add_class("my_class");
    assert_eq!(
        node.to_web_sys().unchecked_into::<Element>().class_name(),
        "my_class"
    );
    node.remove_class("my_class");
    node.add_class("hyphenated-class");
    assert_eq!(
        node.to_web_sys().unchecked_into::<Element>().class_name(),
        "hyphenated-class"
    );
    node.remove_class("hyphenated-class");
    node.add_class("multiple classes");
    assert_eq!(
        node.to_web_sys().unchecked_into::<Element>().class_name(),
        "multiple classes"
    );
}

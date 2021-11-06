use super::*;

#[wasm_bindgen_test]
fn lazy() {
    let node: View<DomNode> = View::new_dyn(|| {
        view! {
            div {
                "Test"
            }
        }
    });

    sycamore::render_to(|| node, &test_container());

    assert_eq!(
        document()
            .query_selector("div")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Test"
    );
}

#[wasm_bindgen_test]
fn lazy_reactive() {
    let template = Signal::new(view! {
        "1"
    });

    let node: View<DomNode> =
        View::new_dyn(cloned!((template) => move || (*template.get()).clone()));

    sycamore::render_to(|| node, &test_container());
    let test_container = document()
        .query_selector("test-container")
        .unwrap()
        .unwrap();

    assert_eq!(test_container.text_content().unwrap(), "1");

    template.set(view! {
        "2"
    });

    assert_eq!(test_container.text_content().unwrap(), "2");
}

#[wasm_bindgen_test]
fn lazy_in_fragment() {
    let num = Signal::new(0);

    let node = cloned!((num) => view! {
        "before"
        p { (num.get()) }
        "after"
    });

    sycamore::render_to(|| node, &test_container());
    let test_container = document()
        .query_selector("test-container")
        .unwrap()
        .unwrap();

    assert_eq!(test_container.text_content().unwrap(), "before0after");

    num.set(1);

    assert_eq!(test_container.text_content().unwrap(), "before1after");
}

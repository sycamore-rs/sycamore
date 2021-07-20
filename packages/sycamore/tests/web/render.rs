use super::*;

#[wasm_bindgen_test]
fn lazy() {
    let _ = create_root(|| {
        let node: Template<DomNode> = Template::new_dyn(|| {
            template! {
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
    });
}

#[wasm_bindgen_test]
fn lazy_reactive() {
    let _ = create_root(|| {
        let (template, set_template) = create_signal(template! {
            "1"
        });

        let node: Template<DomNode> = Template::new_dyn(move || (*template.get()).clone());

        sycamore::render_to(|| node, &test_container());
        let test_container = document()
            .query_selector("test-container")
            .unwrap()
            .unwrap();

        assert_eq!(test_container.text_content().unwrap(), "1");

        set_template.set(template! {
            "2"
        });

        assert_eq!(test_container.text_content().unwrap(), "2");
    });
}

#[wasm_bindgen_test]
fn lazy_in_fragment() {
    let _ = create_root(|| {
        let (num, set_num) = create_signal(0);

        let node = template! {
            "before"
            p { (num.get()) }
            "after"
        };

        sycamore::render_to(|| node, &test_container());
        let test_container = document()
            .query_selector("test-container")
            .unwrap()
            .unwrap();

        assert_eq!(test_container.text_content().unwrap(), "before0after");

        set_num.set(1);

        assert_eq!(test_container.text_content().unwrap(), "before1after");
    });
}

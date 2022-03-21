use super::*;

#[wasm_bindgen_test]
fn lazy() {
    create_scope_immediate(|cx| {
        let node: View<DomNode> = View::new_dyn(cx, move || {
            view! { cx,
                div {
                    "Test"
                }
            }
        });

        sycamore::render_to(|_| node, &test_container());
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
    create_scope_immediate(|cx| {
        let template = create_signal(cx, view! { cx,
            "1"
        });
        let node: View<DomNode> = View::new_dyn(cx, || (*template.get()).clone());

        sycamore::render_to(|_| node, &test_container());
        let test_container = document()
            .query_selector("test-container")
            .unwrap()
            .unwrap();

        assert_eq!(test_container.text_content().unwrap(), "1");

        template.set(view! { cx,
            "2"
        });
        assert_eq!(test_container.text_content().unwrap(), "2");
    });
}

#[wasm_bindgen_test]
fn lazy_in_fragment() {
    create_scope_immediate(|cx| {
        let num = create_signal(cx, 0);

        let node = view! { cx,
            "before"
            p { (num.get()) }
            "after"
        };

        sycamore::render_to(|_| node, &test_container());
        let test_container = document()
            .query_selector("test-container")
            .unwrap()
            .unwrap();

        assert_eq!(test_container.text_content().unwrap(), "before0after");

        num.set(1);

        assert_eq!(test_container.text_content().unwrap(), "before1after");
    });
}

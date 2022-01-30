use super::*;

#[wasm_bindgen_test]
fn lazy() {
    create_scope_immediate(|ctx| {
        let node: View<DomNode> = View::new_dyn(ctx, move || {
            view! { ctx,
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
    create_scope_immediate(|ctx| {
        let template = ctx.create_signal(view! { ctx,
            "1"
        });
        let node: View<DomNode> = View::new_dyn(ctx, || (*template.get()).clone());

        sycamore::render_to(|_| node, &test_container());
        let test_container = document()
            .query_selector("test-container")
            .unwrap()
            .unwrap();

        assert_eq!(test_container.text_content().unwrap(), "1");

        template.set(view! { ctx,
            "2"
        });
        assert_eq!(test_container.text_content().unwrap(), "2");
    });
}

#[wasm_bindgen_test]
fn lazy_in_fragment() {
    create_scope_immediate(|ctx| {
        let num = ctx.create_signal(0);

        let node = view! { ctx,
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

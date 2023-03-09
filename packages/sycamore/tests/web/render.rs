use super::*;

#[wasm_bindgen_test]
fn dyn_view_static() {
    create_scope_immediate(|cx| {
        let node: View<DomNode> = View::new_dyn(cx, move || {
            view! { cx,
                div {
                    "Test"
                }
            }
        });

        sycamore::render_to(|_| node, &test_container());
        assert_text_content!(query("div"), "Test");
    });
}

#[wasm_bindgen_test]
fn dyn_view() {
    create_scope_immediate(|cx| {
        let template = create_signal(
            cx,
            view! { cx,
                "1"
            },
        );
        let node: View<DomNode> = View::new_dyn(cx, || (*template.get()).clone());

        sycamore::render_to(|_| node, &test_container());
        let test_container = query("test-container");

        assert_text_content!(test_container, "1");

        template.set(view! { cx,
            "2"
        });
        assert_text_content!(test_container, "2");
    });
}

#[wasm_bindgen_test]
fn dyn_fragment() {
    create_scope_immediate(|cx| {
        let num = create_signal(cx, 0);

        let node = view! { cx,
            "before"
            p { (num.get()) }
            "after"
        };

        sycamore::render_to(|_| node, &test_container());
        let test_container = query("test-container");

        assert_text_content!(test_container, "before0after");

        num.set(1);

        assert_text_content!(test_container, "before1after");
    });
}

#[wasm_bindgen_test]
fn dyn_nested() {
    create_scope_immediate(|cx| {
        let node: View<DomNode> = View::new_dyn(cx, move || {
            View::new_dyn(cx, move || {
                view! { cx,
                    div {
                        "Test"
                    }
                }
            })
        });

        sycamore::render_to(|_| node, &test_container());
        assert_text_content!(query("div"), "Test");
    });
}

#[wasm_bindgen_test]
fn dyn_scoped_nested() {
    create_scope_immediate(|cx| {
        let num = create_signal(cx, 0);

        let node: View<DomNode> = View::new_dyn_scoped(cx, move |cx| {
            View::new_dyn_scoped(cx, move |cx| {
                view! { cx,
                    div {
                        (num.get())
                    }
                }
            })
        });

        sycamore::render_to(|_| node, &test_container());
        assert_text_content!(query("div"), "0");
        num.set(1);
        assert_text_content!(query("div"), "1");
    });
}

#[wasm_bindgen_test]
fn regression_572() {
    let signal = create_rc_signal(0);

    sycamore::render_to(
        {
            let signal = signal.clone();
            |cx| {
                View::new_dyn_scoped(cx, move |cx| {
                    let signal = signal.clone();
                    View::new_dyn(cx, move || {
                        signal.track();
                        View::empty()
                    })
                })
            }
        },
        &test_container(),
    );
    signal.set(0);
}

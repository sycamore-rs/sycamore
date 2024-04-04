use super::*;

#[wasm_bindgen_test]
fn dyn_view_static() {
    let _ = create_root(|| {
        let node: View = View::new_dyn(move || {
            view! {
                div {
                    "Test"
                }
            }
        });

        sycamore::render_in_scope(|| node, &test_container());
        assert_text_content!(query("div"), "Test");
    });
}

#[wasm_bindgen_test]
fn dyn_view() {
    let _ = create_root(|| {
        let view = create_signal(view! {
            "1"
        });
        let node: View = View::new_dyn(move || view.get_clone());

        sycamore::render_in_scope(|| node, &test_container());
        let test_container = query("test-container");

        assert_text_content!(test_container, "1");

        view.set(view! {
            "2"
        });
        assert_text_content!(test_container, "2");
    });
}

#[wasm_bindgen_test]
fn dyn_fragment() {
    let _ = create_root(|| {
        let num = create_signal(0);

        let node = view! {
            "before"
            p { (num.get()) }
            "after"
        };

        sycamore::render_in_scope(|| node, &test_container());
        let test_container = query("test-container");

        assert_text_content!(test_container, "before0after");

        num.set(1);

        assert_text_content!(test_container, "before1after");
    });
}

#[wasm_bindgen_test]
fn dyn_nested() {
    let _ = create_root(|| {
        let node: View = View::new_dyn(move || {
            View::new_dyn(move || {
                view! {
                    div {
                        "Test"
                    }
                }
            })
        });

        sycamore::render_in_scope(|| node, &test_container());
        assert_text_content!(query("div"), "Test");
    });
}

#[wasm_bindgen_test]
fn dyn_scoped_nested() {
    let _ = create_root(|| {
        let num = create_signal(0);

        let node: View = View::new_dyn(move || {
            View::new_dyn(move || {
                view! {
                    div {
                        (num.get())
                    }
                }
            })
        });

        sycamore::render_in_scope(|| node, &test_container());
        assert_text_content!(query("div"), "0");
        num.set(1);
        assert_text_content!(query("div"), "1");
    });
}

#[wasm_bindgen_test]
fn regression_572() {
    let _ = create_root(|| {
        let signal = create_signal(0);

        sycamore::render_in_scope(
            move || {
                View::new_dyn(move || {
                    View::new_dyn(move || {
                        signal.track();
                        View::empty()
                    })
                })
            },
            &test_container(),
        );
        signal.set(0);
    });
}

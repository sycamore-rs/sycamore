use std::iter::once;

use super::*;

#[wasm_bindgen_test]
fn append() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");

        assert_text_content!(p, "12");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(3);
            tmp
        });
        assert_text_content!(p, "123");

        count.set(count.get()[1..].into());
        assert_text_content!(p, "23");
    });
}

#[wasm_bindgen_test]
fn swap_rows() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2, 3]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.swap(0, 2);
            tmp
        });
        assert_text_content!(p, "321");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.swap(0, 2);
            tmp
        });
        assert_text_content!(p, "123");
    });
}

#[wasm_bindgen_test]
fn update_row() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "12");

        count.set(vec![1, 3]);
        assert_text_content!(p, "13");
    });
}

#[wasm_bindgen_test]
fn trigger_with_same_data() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "12");

        count.set(count.get().as_ref().clone());
        assert_text_content!(p, "12");
    });
}

#[wasm_bindgen_test]
fn delete_row() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2, 3]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.remove(1);
            tmp
        });
        assert_text_content!(p, "13");
    });
}

#[wasm_bindgen_test]
fn delete_row_from_start() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "12");

        count.set(count.get().iter().cloned().skip(1).collect());
        assert_text_content!(p, "2");
    });
}

#[wasm_bindgen_test]
fn delete_row_from_end() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "12");

        count.set(count.get().iter().cloned().take(1).collect());
        assert_text_content!(p, "1");
    });
}

#[wasm_bindgen_test]
fn clear() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2, 3]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.set(Vec::new());
        assert_text_content!(p, "");
    });
}

#[wasm_bindgen_test]
fn insert_front() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2, 3]);

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.insert(0, 4);
            tmp
        });
        assert_text_content!(p, "4123");
    });
}

#[wasm_bindgen_test]
fn nested_reactivity() {
    create_scope_immediate(|cx| {
        let count = create_signal(
            cx,
            vec![1, 2, 3]
                .into_iter()
                .map(|x| create_rc_signal(x))
                .collect(),
        );

        let node = view! { cx,
            ul {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        li { (item.get()) }
                    },
                    key=|item| *item.get(),
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.get()[0].set(4);
        assert_text_content!(p, "423");

        count.modify().push(create_rc_signal(5));
        assert_text_content!(p, "4235");
    });
}

#[wasm_bindgen_test]
fn fragment_template() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);

        let node = view! { cx,
            div {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        span { "The value is: " }
                        strong { (item) }
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("div");

        assert_text_content!(
            p,
            "\
    The value is: 1\
    The value is: 2"
        );

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(3);
            tmp
        });
        assert_text_content!(
            p,
            "\
    The value is: 1\
    The value is: 2\
    The value is: 3"
        );

        count.set(count.get()[1..].into());
        assert_text_content!(
            p,
            "\
    The value is: 2\
    The value is: 3"
        );
    });
}

#[wasm_bindgen_test]
fn template_top_level() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);

        let node = view! { cx,
            Keyed(
                iterable=count,
                view=|cx, item| view! { cx,
                    li { (item) }
                },
                key=|item| *item,
            )
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("#test-container");

        assert_text_content!(p, "12");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(3);
            tmp
        });
        assert_text_content!(p, "123");

        count.set(count.get()[1..].into());
        assert_text_content!(p, "23");
    });
}

#[wasm_bindgen_test]
fn template_dyn_top_level() {
    create_scope_immediate(|cx| {
        let count = create_signal(cx, vec![1, 2]);

        let node = view! { cx,
            div {
                Keyed(
                    iterable=count,
                    view=|cx, item| view! { cx,
                        (item)
                    },
                    key=|item| *item,
                )
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = query("#test-container");

        assert_text_content!(p, "12");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(3);
            tmp
        });
        assert_text_content!(p, "123");

        count.set(count.get()[1..].into());
        assert_text_content!(p, "23");
    });
}

#[wasm_bindgen_test]
fn template_with_other_nodes_at_same_level() {
    create_scope_immediate(|cx| {
        let vec1 = create_signal(cx, vec![1, 2]);
        let vec2 = create_signal(cx, vec![4, 5]);

        let node = view! { cx,
            ul {
                li { "before" }
                Keyed(
                    iterable=vec1,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|x| *x
                )
                Keyed(
                    iterable=vec2,
                    view=|cx, item| view! { cx,
                        li { (item) }
                    },
                    key=|x| *x
                )
                li { "after" }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let elem = query("ul");

        assert_text_content!(elem, "before1245after");

        vec1.set(vec1.get().iter().cloned().chain(once(3)).collect());
        assert_text_content!(elem, "before12345after");

        vec1.set(Vec::new());
        assert_text_content!(elem, "before45after");

        vec1.set(vec![1]);
        assert_text_content!(elem, "before145after");
    });
}

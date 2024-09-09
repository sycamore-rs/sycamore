use super::*;

#[wasm_bindgen_test]
fn append() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");

        assert_text_content!(p, "12");

        count.update(|count| count.push(3));
        assert_text_content!(p, "123");

        count.update(|count| count.remove(0));
        assert_text_content!(p, "23");
    });
}

#[wasm_bindgen_test]
fn swap_rows() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2, 3]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.update(|count| count.swap(0, 2));
        assert_text_content!(p, "321");

        count.update(|count| count.swap(0, 2));
        assert_text_content!(p, "123");
    });
}

#[wasm_bindgen_test]
fn update_row() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "12");

        count.set(vec![1, 3]);
        assert_text_content!(p, "13");
    });
}

#[wasm_bindgen_test]
fn trigger_with_same_data() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "12");

        count.update(|_| {});
        assert_text_content!(p, "12");
    });
}

#[wasm_bindgen_test]
fn delete_row() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2, 3]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.update(|count| count.remove(1));
        assert_text_content!(p, "13");
    });
}

#[wasm_bindgen_test]
fn delete_row_from_start() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "12");

        count.update(|count| count.remove(0));
        assert_text_content!(p, "2");
    });
}

#[wasm_bindgen_test]
fn delete_row_from_end() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "12");

        count.update(|count| count.truncate(1));
        assert_text_content!(p, "1");
    });
}

#[wasm_bindgen_test]
fn clear() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2, 3]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.set(Vec::new());
        assert_text_content!(p, "");
    });
}

#[wasm_bindgen_test]
fn insert_front() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2, 3]);

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.update(|count| count.insert(0, 4));
        assert_text_content!(p, "4123");
    });
}

#[wasm_bindgen_test]
fn nested_reactivity() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2, 3].into_iter().map(create_signal).collect());

        let node = view! {
            ul {
                Indexed(
                    list=count,
                    view=|item| view! {
                        li { (item.get()) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("ul");
        assert_text_content!(p, "123");

        count.get_clone()[0].set(4);
        assert_text_content!(p, "423");

        let new = create_signal(5);
        count.update(|count| count.push(new));
        assert_text_content!(p, "4235");
    });
}

#[wasm_bindgen_test]
fn fragment_template() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);

        let node = view! {
            div {
                Indexed(
                    list=count,
                    view=|item| view! {
                        span { "The value is: " }
                        strong { (item) }
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("div");

        assert_text_content!(
            p,
            "\
    The value is: 1\
    The value is: 2"
        );

        count.update(|count| count.push(3));
        assert_text_content!(
            p,
            "\
    The value is: 1\
    The value is: 2\
    The value is: 3"
        );

        count.update(|count| count.remove(0));
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
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);

        let node = view! {
            Indexed(
                list=count,
                view=|item| view! {
                    li { (item) }
                },
            )
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("#test-container");

        assert_text_content!(p, "12");

        count.update(|count| count.push(3));
        assert_text_content!(p, "123");

        count.update(|count| count.remove(0));
        assert_text_content!(p, "23");
    });
}

#[wasm_bindgen_test]
fn template_dyn_top_level() {
    let _ = create_root(|| {
        let count = create_signal(vec![1, 2]);

        let node = view! {
            div {
                Indexed(
                    list=count,
                    view=|item| view! {
                        (item)
                    },
                )
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let p = query("#test-container");

        assert_text_content!(p, "12");

        count.update(|count| count.push(3));
        assert_text_content!(p, "123");

        count.update(|count| count.remove(0));
        assert_text_content!(p, "23");
    });
}

#[wasm_bindgen_test]
fn template_with_other_nodes_at_same_level() {
    let _ = create_root(|| {
        let vec1 = create_signal(vec![1, 2]);
        let vec2 = create_signal(vec![4, 5]);

        let node = view! {
            ul {
                li { "before" }
                Indexed(
                    list=vec1,
                    view=|item| view! {
                        li { (item) }
                    },
                )
                Indexed(
                    list=vec2,
                    view=|item| view! {
                        li { (item) }
                    },
                )
                li { "after" }
            }
        };

        sycamore::render_in_scope(|| node, &test_container());

        let elem = query("ul");

        assert_text_content!(elem, "before1245after");

        vec1.update(|vec1| vec1.push(3));
        assert_text_content!(elem, "before12345after");

        vec1.set(Vec::new());
        assert_text_content!(elem, "before45after");

        vec1.set(vec![1]);
        assert_text_content!(elem, "before145after");
    });
}

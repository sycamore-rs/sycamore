use std::iter::once;

use super::*;

#[wasm_bindgen_test]
fn append() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();

        assert_eq!(p.text_content().unwrap(), "0112");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(3);
            tmp
        });
        assert_eq!(p.text_content().unwrap(), "011223");

        count.set(count.get()[1..].into());
        assert_eq!(p.text_content().unwrap(), "0213");
    });
}

#[wasm_bindgen_test]
fn swap_rows() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2, 3]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "011223");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.swap(0, 2);
            tmp
        });
        assert_eq!(p.text_content().unwrap(), "031221");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.swap(0, 2);
            tmp
        });
        assert_eq!(p.text_content().unwrap(), "011223");
    });
}

#[wasm_bindgen_test]
fn update_row() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "0112");

        count.set(vec![1, 3]);
        assert_eq!(p.text_content().unwrap(), "0113");
    });
}

#[wasm_bindgen_test]
fn trigger_with_same_data() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "0112");

        count.set(count.get().as_ref().clone());
        assert_eq!(p.text_content().unwrap(), "0112");
    });
}

#[wasm_bindgen_test]
fn delete_row() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2, 3]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "011223");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.remove(1);
            tmp
        });
        assert_eq!(p.text_content().unwrap(), "0113");
    });
}

#[wasm_bindgen_test]
fn delete_row_from_start() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "0112");

        count.set(count.get().iter().cloned().skip(1).collect());
        assert_eq!(p.text_content().unwrap(), "02");
    });
}

#[wasm_bindgen_test]
fn delete_row_from_end() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "0112");

        count.set(count.get().iter().cloned().take(1).collect());
        assert_eq!(p.text_content().unwrap(), "01");
    });
}

#[wasm_bindgen_test]
fn clear() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2, 3]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "011223");

        count.set(Vec::new());
        assert_eq!(p.text_content().unwrap(), "");
    });
}

#[wasm_bindgen_test]
fn insert_front() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2, 3]);

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "011223");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.insert(0, 4);
            tmp
        });
        assert_eq!(p.text_content().unwrap(), "04112233");
    });
}

#[wasm_bindgen_test]
fn nested_reactivity() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(
            vec![1, 2, 3]
                .into_iter()
                .map(|x| ctx.create_signal(x))
                .collect(),
        );

        let node = view! { ctx,
            ul {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item.get())) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("ul").unwrap().unwrap();
        assert_eq!(p.text_content().unwrap(), "011223");

        count.get()[0].set(4);
        assert_eq!(p.text_content().unwrap(), "041223");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(ctx.create_signal(5));
            tmp
        });
        assert_eq!(p.text_content().unwrap(), "04122335");
    });
}

#[wasm_bindgen_test]
fn fragment_template() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);

        let node = view! { ctx,
            div {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        span { "The value is: " }
                        strong { (format!("{}{}", idx, item)) }
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document().query_selector("div").unwrap().unwrap();

        assert_eq!(
            p.text_content().unwrap(),
            "\
    The value is: 01\
    The value is: 12"
        );

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(3);
            tmp
        });
        assert_eq!(
            p.text_content().unwrap(),
            "\
    The value is: 01\
    The value is: 12\
    The value is: 23"
        );

        count.set(count.get()[1..].into());
        assert_eq!(
            p.text_content().unwrap(),
            "\
    The value is: 02\
    The value is: 13"
        );
    });
}

#[wasm_bindgen_test]
fn template_top_level() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);

        let node = view! { ctx,
            Indexed {
                iterable: count,
                view: |ctx, item, idx| view! { ctx,
                    li { (format!("{}{}", idx, item)) }
                },
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document()
            .query_selector("#test-container")
            .unwrap()
            .unwrap();

        assert_eq!(p.text_content().unwrap(), "0112");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(3);
            tmp
        });
        assert_eq!(p.text_content().unwrap(), "011223");

        count.set(count.get()[1..].into());
        assert_eq!(p.text_content().unwrap(), "0213");
    });
}

#[wasm_bindgen_test]
fn template_dyn_top_level() {
    create_scope_immediate(|ctx| {
        let count = ctx.create_signal(vec![1, 2]);

        let node = view! { ctx,
            div {
                Indexed {
                    iterable: count,
                    view: |ctx, item, idx| view! { ctx,
                        (format!("{}{}", idx, item))
                    },
                }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let p = document()
            .query_selector("#test-container")
            .unwrap()
            .unwrap();

        assert_eq!(p.text_content().unwrap(), "0112");

        count.set({
            let mut tmp = (*count.get()).clone();
            tmp.push(3);
            tmp
        });
        assert_eq!(p.text_content().unwrap(), "011223");

        count.set(count.get()[1..].into());
        assert_eq!(p.text_content().unwrap(), "0213");
    });
}

#[wasm_bindgen_test]
fn template_with_other_nodes_at_same_level() {
    create_scope_immediate(|ctx| {
        let vec1 = ctx.create_signal(vec![1, 2]);
        let vec2 = ctx.create_signal(vec![4, 5]);

        let node = view! { ctx,
            ul {
                li { "before" }
                Indexed {
                    iterable: vec1,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
                Indexed {
                    iterable: vec2,
                    view: |ctx, item, idx| view! { ctx,
                        li { (format!("{}{}", idx, item)) }
                    },
                }
                li { "after" }
            }
        };

        sycamore::render_to(|_| node, &test_container());

        let elem = document().query_selector("ul").unwrap().unwrap();

        assert_eq!(elem.text_content().unwrap(), "before01120415after");

        vec1.set(vec1.get().iter().cloned().chain(once(3)).collect());
        assert_eq!(elem.text_content().unwrap(), "before0112230415after");

        vec1.set(Vec::new());
        assert_eq!(elem.text_content().unwrap(), "before0415after");

        vec1.set(vec![1]);
        assert_eq!(elem.text_content().unwrap(), "before010415after");
    });
}

use std::iter::once;

use super::*;

#[wasm_bindgen_test]
fn append() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();

    assert_eq!(p.text_content().unwrap(), "12");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.push(3);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "123");

    count.set(count.get()[1..].into());
    assert_eq!(p.text_content().unwrap(), "23");
}

#[wasm_bindgen_test]
fn swap_rows() {
    let count = Signal::new(vec![1, 2, 3]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "123");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.swap(0, 2);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "321");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.swap(0, 2);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "123");
}

#[wasm_bindgen_test]
fn update_row() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "12");

    count.set(vec![1, 3]);
    assert_eq!(p.text_content().unwrap(), "13");
}

#[wasm_bindgen_test]
fn trigger_with_same_data() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "12");

    count.set(count.get().as_ref().clone());
    assert_eq!(p.text_content().unwrap(), "12");
}

#[wasm_bindgen_test]
fn delete_row() {
    let count = Signal::new(vec![1, 2, 3]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "123");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.remove(1);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "13");
}

#[wasm_bindgen_test]
fn delete_row_from_start() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "12");

    count.set(count.get().iter().cloned().skip(1).collect());
    assert_eq!(p.text_content().unwrap(), "2");
}

#[wasm_bindgen_test]
fn delete_row_from_end() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "12");

    count.set(count.get().iter().cloned().take(1).collect());
    assert_eq!(p.text_content().unwrap(), "1");
}

#[wasm_bindgen_test]
fn clear() {
    let count = Signal::new(vec![1, 2, 3]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "123");

    count.set(Vec::new());
    assert_eq!(p.text_content().unwrap(), "");
}

#[wasm_bindgen_test]
fn insert_front() {
    let count = Signal::new(vec![1, 2, 3]);

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "123");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.insert(0, 4);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "4123");
}

#[wasm_bindgen_test]
fn nested_reactivity() {
    let count = Signal::new(vec![1, 2, 3].into_iter().map(Signal::new).collect());

    let node = cloned!((count) => view! {
        ul {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    li { (item.get()) }
                },
                key: |item| *item.get(),
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "123");

    count.get()[0].set(4);
    assert_eq!(p.text_content().unwrap(), "423");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.push(Signal::new(5));
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "4235");
}

#[wasm_bindgen_test]
fn fragment_template() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => view! {
        div {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    span { "The value is: " }
                    strong { (item) }
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document().query_selector("div").unwrap().unwrap();

    assert_eq!(
        p.text_content().unwrap(),
        "\
The value is: 1\
The value is: 2"
    );

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.push(3);
        tmp
    });
    assert_eq!(
        p.text_content().unwrap(),
        "\
The value is: 1\
The value is: 2\
The value is: 3"
    );

    count.set(count.get()[1..].into());
    assert_eq!(
        p.text_content().unwrap(),
        "\
The value is: 2\
The value is: 3"
    );
}

#[wasm_bindgen_test]
fn template_top_level() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => view! {
        Keyed(KeyedProps {
            iterable: count.handle(),
            template: |item| view! {
                li { (item) }
            },
            key: |item| *item,
        })
    });

    sycamore::render_to(|| node, &test_container());

    let p = document()
        .query_selector("#test-container")
        .unwrap()
        .unwrap();

    assert_eq!(p.text_content().unwrap(), "12");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.push(3);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "123");

    count.set(count.get()[1..].into());
    assert_eq!(p.text_content().unwrap(), "23");
}

#[wasm_bindgen_test]
fn template_dyn_top_level() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => view! {
        div {
            Keyed(KeyedProps {
                iterable: count.handle(),
                template: |item| view! {
                    (item)
                },
                key: |item| *item,
            })
        }
    });

    sycamore::render_to(|| node, &test_container());

    let p = document()
        .query_selector("#test-container")
        .unwrap()
        .unwrap();

    assert_eq!(p.text_content().unwrap(), "12");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.push(3);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "123");

    count.set(count.get()[1..].into());
    assert_eq!(p.text_content().unwrap(), "23");
}

#[wasm_bindgen_test]
fn template_with_other_nodes_at_same_level() {
    let vec1 = Signal::new(vec![1, 2]);
    let vec2 = Signal::new(vec![4, 5]);

    let node = cloned!((vec1, vec2) => view! {
        ul {
            li { "before" }
            Keyed(KeyedProps {
                iterable: vec1.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |x| *x
            })
            Keyed(KeyedProps {
                iterable: vec2.handle(),
                template: |item| view! {
                    li { (item) }
                },
                key: |x| *x
            })
            li { "after" }
        }
    });

    sycamore::render_to(|| node, &test_container());

    let elem = document().query_selector("ul").unwrap().unwrap();

    assert_eq!(elem.text_content().unwrap(), "before1245after");

    vec1.set(vec1.get().iter().cloned().chain(once(3)).collect());
    assert_eq!(elem.text_content().unwrap(), "before12345after");

    vec1.set(Vec::new());
    assert_eq!(elem.text_content().unwrap(), "before45after");

    vec1.set(vec![1]);
    assert_eq!(elem.text_content().unwrap(), "before145after");
}

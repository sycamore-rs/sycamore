use super::*;

#[wasm_bindgen_test]
fn append() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => template! {
        ul {
            Indexed<_, _>(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_container());

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

    let node = cloned!((count) => template! {
        ul {
            Indexed<_, _>(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_container());

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
fn delete_row() {
    let count = Signal::new(vec![1, 2, 3]);

    let node = cloned!((count) => template! {
        ul {
            Indexed<_, _>(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_container());

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
fn clear() {
    let count = Signal::new(vec![1, 2, 3]);

    let node = cloned!((count) => template! {
        ul {
            Indexed<_, _>(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_container());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "123");

    count.set(Vec::new());
    assert_eq!(p.text_content().unwrap(), "");
}

#[wasm_bindgen_test]
fn insert_front() {
    let count = Signal::new(vec![1, 2, 3]);

    let node = cloned!((count) => template! {
        ul {
            Indexed<_, _>(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_container());

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

    let node = cloned!((count) => template! {
        ul {
            Indexed<_, _>(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item.get()) }
                },
            })
        }
    });

    render_to(|| node, &test_container());

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

    let node = cloned!((count) => template! {
        div {
            Indexed<_, _>(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    span { "The value is: " }
                    strong { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_container());

    let p = document().query_selector("div").unwrap().unwrap();

    assert_eq!(
        p.inner_html(),
        "\
<span>The value is: </span><strong>1</strong>\
<span>The value is: </span><strong>2</strong>\
<!---->"
    );

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.push(3);
        tmp
    });
    assert_eq!(
        p.inner_html(),
        "\
<span>The value is: </span><strong>1</strong>\
<span>The value is: </span><strong>2</strong>\
<span>The value is: </span><strong>3</strong>\
<!---->"
    );

    count.set(count.get()[1..].into());
    assert_eq!(
        p.inner_html(),
        "\
<span>The value is: </span><strong>2</strong>\
<span>The value is: </span><strong>3</strong>\
<!---->"
    );
}

#[wasm_bindgen_test]
fn template_top_level() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => template! {
        Indexed<_, _>(IndexedProps {
            iterable: count.handle(),
            template: |item| template! {
                li { (item) }
            },
        })
    });

    render_to(|| node, &test_container());

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

use super::*;

#[wasm_bindgen_test]
fn append() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => template! {
        ul {
            Indexed(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_div());

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
            Indexed(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_div());

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
fn insert_front() {
    let count = Signal::new(vec![1, 2, 3]);

    let node = cloned!((count) => template! {
        ul {
            Indexed(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_div());

    let p = document().query_selector("ul").unwrap().unwrap();
    assert_eq!(p.text_content().unwrap(), "123");

    count.set({
        let mut tmp = (*count.get()).clone();
        tmp.insert(0, 4);
        tmp
    });
    assert_eq!(p.text_content().unwrap(), "4123");
}

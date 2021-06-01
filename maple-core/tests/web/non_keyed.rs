use ::std::vec::Vec;
use std::iter::once;

use ::maple_core::generic_node::{render, GenericNode};
use ::maple_core::reactive;
use ::maple_core::render::Render;
use maple_core::template_result::TemplateResult;

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
            Indexed(IndexedProps {
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
            Indexed(IndexedProps {
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
            Indexed(IndexedProps {
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
            Indexed(IndexedProps {
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

    let node_ref = NodeRef::new();

    // let node = cloned!((count) => template! {
    //     ul {
    //         Indexed(IndexedProps {
    //             iterable: count.handle(),
    //             template: cloned!((node_ref) => move |item| template! {
    //                 li(ref=node_ref) { (item.get()) }
    //             })
    //         })
    //     }
    // });
    let node = {
        let count = count.clone();
        TemplateResult::new_node({
            let element = GenericNode::element("ul");
            render::insert(
                Clone::clone(&element),
                reactive::untrack(|| {
                    Indexed::<_>::__create_component(IndexedProps {
                        iterable: count.handle(),
                        template: {
                            let node_ref = Clone::clone(&node_ref);
                            move |item| {
                                TemplateResult::new_node({
                                    let element = DomNode::element("li");
                                    web_sys::console::log_1(&element.inner_element());
                                    NodeRef::set(&node_ref, element.clone());
                                    render::insert(
                                        element.clone(),
                                        TemplateResult::new_lazy(move || {
                                            let mut nodes = Render::create(&item.get());
                                            if nodes.len() == 1 {
                                                TemplateResult::new_node(nodes.remove(0))
                                            } else {
                                                let nodes = nodes
                                                    .into_iter()
                                                    .map(TemplateResult::new_node)
                                                    .collect();
                                                TemplateResult::new_fragment(nodes)
                                            }
                                        }),
                                        None,
                                        None,
                                    );
                                    element
                                })
                            }
                        },
                    })
                }),
                None,
                None,
            );
            element
        })
    };

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
            Indexed(IndexedProps {
                iterable: count.handle(),
                template: |item| template! {
                    span { "The value is: " }
                    strong { (item) }
                },
            })
        }
    });

    render_to(|| node, &test_container());

    let elem = document().query_selector("div").unwrap().unwrap();

    assert_eq!(
        elem.text_content().unwrap(),
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
        elem.text_content().unwrap(),
        "\
The value is: 1\
The value is: 2\
The value is: 3"
    );

    count.set(count.get()[1..].into());
    assert_eq!(
        elem.text_content().unwrap(),
        "\
The value is: 2\
The value is: 3"
    );
}

#[wasm_bindgen_test]
fn template_top_level() {
    let count = Signal::new(vec![1, 2]);

    let node = cloned!((count) => template! {
        Indexed(IndexedProps {
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

#[wasm_bindgen_test]
fn template_with_other_nodes_at_same_level() {
    let vec1 = Signal::new(vec![1, 2]);
    let vec2 = Signal::new(vec![4, 5]);

    let node = cloned!((vec1, vec2) => template! {
        ul {
            li { "before" }
            Indexed(IndexedProps {
                iterable: vec1.handle(),
                template: |item| {
                    web_sys::console::log_1(&"rendered".into());
                    template! {
                        (item)
                    }
                },
            })
            Indexed(IndexedProps {
                iterable: vec2.handle(),
                template: |item| template! {
                    li { (item) }
                },
            })
            li { "after" }
        }
    });

    render_to(|| node, &test_container());

    let elem = document().query_selector("ul").unwrap().unwrap();

    assert_eq!(elem.text_content().unwrap(), "before1245after");

    vec1.set(vec1.get().iter().cloned().chain(once(3)).collect());
    assert_eq!(elem.text_content().unwrap(), "before12345after");

    vec1.set(Vec::new());
    assert_eq!(elem.text_content().unwrap(), "before45after");

    vec1.set(vec![1]);
    assert_eq!(elem.text_content().unwrap(), "before145after");
}

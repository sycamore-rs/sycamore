//! Tests for [`reconcile_fragments`].

use maple_core::generic_node::render::{append_nodes, insert, reconcile_fragments};
use maple_core::generic_node::DomNode;

use super::*;

use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn insert_create_nodes() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let parent = DomNode::element("div");

    insert(
        parent.clone(),
        TemplateResult::new_fragment(
            nodes
                .to_vec()
                .into_iter()
                .map(TemplateResult::new_node)
                .collect(),
        ),
        None,
        None,
    );
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");
}

#[wasm_bindgen_test]
fn reconcile_pop_nodes() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let parent = DomNode::element("div");
    let child_nodes = nodes.to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");

    reconcile_fragments(parent.clone(), child_nodes, nodes[..2].to_vec());
    assert_eq!(parent.inner_element().text_content().unwrap(), "12");
}

#[wasm_bindgen_test]
fn reconcile_remove_nodes() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let parent = DomNode::element("div");
    let child_nodes = nodes.to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");

    reconcile_fragments(
        parent.clone(),
        child_nodes,
        vec![nodes[0].clone(), nodes[2].clone()],
    );
    assert_eq!(parent.inner_element().text_content().unwrap(), "13");
}

#[wasm_bindgen_test]
fn reconcile_append_nodes() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let parent = DomNode::element("div");
    let child_nodes = nodes[..2].to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_eq!(parent.inner_element().text_content().unwrap(), "12");

    reconcile_fragments(parent.clone(), child_nodes, nodes.to_vec());
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");
}

#[wasm_bindgen_test]
fn reconcile_swap_nodes() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let parent = DomNode::element("div");
    let child_nodes = nodes.to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");

    reconcile_fragments(
        parent.clone(),
        child_nodes,
        vec![nodes[2].clone(), nodes[1].clone(), nodes[0].clone()],
    );
    assert_eq!(parent.inner_element().text_content().unwrap(), "321");
}

#[wasm_bindgen_test]
fn reconcile_clear_nodes() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let parent = DomNode::element("div");
    let child_nodes = nodes.to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");

    reconcile_fragments(parent.clone(), child_nodes, Vec::new());
    assert_eq!(parent.inner_element().text_content().unwrap(), "");
}

#[wasm_bindgen_test]
fn clear_and_insert_with_other_nodes_at_same_level() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let before = DomNode::text_node("before");
    let after = DomNode::text_node("after");
    let parent = DomNode::element("div");
    parent.append_child(&before);
    let child_nodes = nodes.to_vec();
    for node in &child_nodes {
        parent.append_child(node);
    }
    parent.append_child(&after);

    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "before123after"
    );

    reconcile_fragments(parent.clone(), child_nodes.clone(), Vec::new());
    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "beforeafter"
    );

    append_nodes(&parent, child_nodes, Some(after));
    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "before123after"
    );
}

#[wasm_bindgen_test]
fn clear_with_other_nodes_at_same_level() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let before = DomNode::text_node("before");
    let after = DomNode::text_node("after");
    let parent = DomNode::element("div");
    parent.append_child(&before);
    let child_nodes = nodes.to_vec();
    for node in &child_nodes {
        parent.append_child(node);
    }
    parent.append_child(&after);

    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "before123after"
    );

    reconcile_fragments(parent.clone(), child_nodes, Vec::new());
    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "beforeafter"
    );
}

#[wasm_bindgen_test]
fn insert_with_other_nodes_at_same_level() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let before = DomNode::text_node("before");
    let after = DomNode::text_node("after");
    let parent = DomNode::element("div");
    parent.append_child(&before);
    let child_nodes = nodes.to_vec();
    for node in &child_nodes {
        parent.append_child(node);
    }
    parent.append_child(&after);

    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "before123after"
    );

    reconcile_fragments(
        parent.clone(),
        child_nodes.clone(),
        child_nodes[..2].to_vec(),
    );
    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "before12after"
    );
}

#[wasm_bindgen_test]
fn reconcile_with_other_nodes_at_same_level() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let before = DomNode::text_node("before");
    let after = DomNode::text_node("after");
    let parent = DomNode::element("div");
    parent.append_child(&before);
    let child_nodes = nodes.to_vec();
    for node in &child_nodes {
        parent.append_child(node);
    }
    parent.append_child(&after);

    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "before123after"
    );

    reconcile_fragments(
        parent.clone(),
        child_nodes.clone(),
        child_nodes[..2].to_vec(),
    );
    assert_eq!(
        parent.inner_element().text_content().unwrap(),
        "before12after"
    );
}

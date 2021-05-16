//! Tests for [`reconcile_fragments`].

use maple_core::generic_node::render::reconcile_fragments;
use maple_core::generic_node::DomNode;

use super::*;

use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn reconcile_create_nodes() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let parent = DomNode::element("div");

    reconcile_fragments(parent.clone(), Vec::new(), nodes.to_vec());
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

    reconcile_fragments(parent.clone(), Vec::new(), child_nodes.clone());
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

    reconcile_fragments(parent.clone(), Vec::new(), child_nodes.clone());
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

    reconcile_fragments(parent.clone(), Vec::new(), child_nodes.clone());
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

    reconcile_fragments(parent.clone(), Vec::new(), child_nodes.clone());
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");

    reconcile_fragments(
        parent.clone(),
        child_nodes,
        vec![nodes[2].clone(), nodes[1].clone(), nodes[0].clone()],
    );
    assert_eq!(parent.inner_element().text_content().unwrap(), "321");
}

#[wasm_bindgen_test]
fn reconcile_do_not_clone_node() {
    let nodes = [
        DomNode::text_node("1"),
        DomNode::text_node("2"),
        DomNode::text_node("3"),
    ];
    let parent = DomNode::element("div");

    reconcile_fragments(parent.clone(), Vec::new(), nodes.to_vec());
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");

    nodes[0].inner_element().set_text_content(Some("4"));
    assert_eq!(parent.inner_element().text_content().unwrap(), "423");
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

    reconcile_fragments(parent.clone(), Vec::new(), child_nodes.clone());
    assert_eq!(parent.inner_element().text_content().unwrap(), "123");

    reconcile_fragments(parent.clone(), child_nodes, Vec::new());
    assert_eq!(parent.inner_element().text_content().unwrap(), "");
}

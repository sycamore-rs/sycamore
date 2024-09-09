//! Tests for [`reconcile_fragments`].

#![allow(clippy::redundant_clone)] // Borrow checking error

use sycamore::utils::render::{append_nodes, insert, reconcile_fragments};
use sycamore_web::tags::*;
use sycamore_web::{IntoHtmlNode, ViewNode};
use wasm_bindgen_test::*;

use super::*;

macro_rules! nodes {
    ($($node:expr),*) => {
        [$(
            HtmlNode::create_text_node(stringify!($node).into()),
        )*]
    };
}

#[wasm_bindgen_test]
fn reconcile_pop_nodes() {
    let nodes = nodes![1, 2, 3];
    let parent = div().into_html_node();

    parent.append_view(View::from_nodes(nodes));
    assert_text_content!(parent.to_web_sys(), "123");

    reconcile_fragments(&parent, &mut child_nodes.clone(), &nodes[..2]);
    assert_text_content!(parent.to_web_sys(), "12");
}

#[wasm_bindgen_test]
fn reconcile_remove_nodes() {
    let nodes = [
        DomNode::text_node("1".into()),
        DomNode::text_node("2".into()),
        DomNode::text_node("3".into()),
    ];
    let parent = DomNode::element::<html::div>();
    let child_nodes = nodes.to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_text_content!(parent.to_web_sys(), "123");

    reconcile_fragments(
        &parent,
        &mut child_nodes.clone(),
        &[nodes[0].clone(), nodes[2].clone()],
    );
    assert_text_content!(parent.to_web_sys(), "13");
}

#[wasm_bindgen_test]
fn reconcile_append_nodes() {
    let nodes = [
        DomNode::text_node("1".into()),
        DomNode::text_node("2".into()),
        DomNode::text_node("3".into()),
    ];
    let parent = DomNode::element::<html::div>();
    let child_nodes = nodes[..2].to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_text_content!(parent.to_web_sys(), "12");

    reconcile_fragments(&parent, &mut child_nodes.clone(), &nodes);
    assert_text_content!(parent.to_web_sys(), "123");
}

#[wasm_bindgen_test]
fn reconcile_swap_nodes() {
    let nodes = [
        DomNode::text_node("1".into()),
        DomNode::text_node("2".into()),
        DomNode::text_node("3".into()),
    ];
    let parent = DomNode::element::<html::div>();
    let child_nodes = nodes.to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_text_content!(parent.to_web_sys(), "123");

    reconcile_fragments(
        &parent,
        &mut child_nodes.clone(),
        &[nodes[2].clone(), nodes[1].clone(), nodes[0].clone()],
    );
    assert_text_content!(parent.to_web_sys(), "321");
}

#[wasm_bindgen_test]
fn reconcile_clear_nodes() {
    let nodes = [
        DomNode::text_node("1".into()),
        DomNode::text_node("2".into()),
        DomNode::text_node("3".into()),
    ];
    let parent = DomNode::element::<html::div>();
    let child_nodes = nodes.to_vec();

    for node in &child_nodes {
        parent.append_child(node);
    }
    assert_text_content!(parent.to_web_sys(), "123");

    reconcile_fragments(&parent, &mut child_nodes.clone(), &[]);
    assert_text_content!(parent.to_web_sys(), "");
}

#[wasm_bindgen_test]
fn clear_and_insert_with_other_nodes_at_same_level() {
    let nodes = [
        DomNode::text_node("1".into()),
        DomNode::text_node("2".into()),
        DomNode::text_node("3".into()),
    ];
    let before = DomNode::text_node("before".into());
    let after = DomNode::text_node("after".into());
    let parent = DomNode::element::<html::div>();
    parent.append_child(&before);
    let child_nodes = nodes.to_vec();
    for node in &child_nodes {
        parent.append_child(node);
    }
    parent.append_child(&after);

    assert_text_content!(parent.to_web_sys(), "before123after");

    reconcile_fragments(&parent, &mut child_nodes.clone(), &[]);
    assert_text_content!(parent.to_web_sys(), "beforeafter");

    append_nodes(&parent, child_nodes, Some(&after));
    assert_text_content!(parent.to_web_sys(), "before123after");
}

#[wasm_bindgen_test]
fn clear_with_other_nodes_at_same_level() {
    let nodes = [
        DomNode::text_node("1".into()),
        DomNode::text_node("2".into()),
        DomNode::text_node("3".into()),
    ];
    let before = DomNode::text_node("before".into());
    let after = DomNode::text_node("after".into());
    let parent = DomNode::element::<html::div>();
    parent.append_child(&before);
    let child_nodes = nodes.to_vec();
    for node in &child_nodes {
        parent.append_child(node);
    }
    parent.append_child(&after);

    assert_text_content!(parent.to_web_sys(), "before123after");

    reconcile_fragments(&parent, &mut child_nodes.clone(), &[]);
    assert_text_content!(parent.to_web_sys(), "beforeafter");
}

#[wasm_bindgen_test]
fn insert_with_other_nodes_at_same_level() {
    let nodes = [
        DomNode::text_node("1".into()),
        DomNode::text_node("2".into()),
        DomNode::text_node("3".into()),
    ];
    let before = DomNode::text_node("before".into());
    let after = DomNode::text_node("after".into());
    let parent = DomNode::element::<html::div>();
    parent.append_child(&before);
    let child_nodes = nodes.to_vec();
    for node in &child_nodes {
        parent.append_child(node);
    }
    parent.append_child(&after);

    assert_text_content!(parent.to_web_sys(), "before123after");

    reconcile_fragments(&parent, &mut child_nodes.clone(), &child_nodes[..2]);
    assert_text_content!(parent.to_web_sys(), "before12after");
}

#[wasm_bindgen_test]
fn reconcile_with_other_nodes_at_same_level() {
    let nodes = [
        DomNode::text_node("1".into()),
        DomNode::text_node("2".into()),
        DomNode::text_node("3".into()),
    ];
    let before = DomNode::text_node("before".into());
    let after = DomNode::text_node("after".into());
    let parent = DomNode::element::<html::div>();
    parent.append_child(&before);
    let child_nodes = nodes.to_vec();
    for node in &child_nodes {
        parent.append_child(node);
    }
    parent.append_child(&after);

    assert_text_content!(parent.to_web_sys(), "before123after");

    reconcile_fragments(&parent, &mut child_nodes.clone(), &child_nodes[..2]);
    assert_text_content!(parent.to_web_sys(), "before12after");
}

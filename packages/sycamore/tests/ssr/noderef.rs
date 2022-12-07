use sycamore::generic_node::GenericNodeElements;
use sycamore::prelude::*;
use sycamore::web::html;

#[test]
fn empty_noderef() {
    let noderef = NodeRef::<SsrNode>::new();
    assert!(noderef.try_get_raw().is_none());
    assert!(noderef.try_get::<SsrNode>().is_none());
}

#[test]
fn set_noderef() {
    let noderef = NodeRef::<SsrNode>::new();
    let node = SsrNode::element::<html::div>();
    noderef.set(node.clone());
    assert_eq!(noderef.try_get_raw(), Some(node.clone()));
    assert_eq!(noderef.try_get::<SsrNode>(), Some(node));
}

#[test]
fn cast_noderef() {
    let noderef = NodeRef::<SsrNode>::new();
    let node = SsrNode::element::<html::div>();
    noderef.set(node.clone());
    assert_eq!(noderef.try_get::<SsrNode>(), Some(node));
    assert!(noderef.try_get::<DomNode>().is_none());
}

#[test]
fn noderef_with_ssrnode() {
    create_scope_immediate(|cx| {
        let noderef = create_node_ref(cx);
        let _: View<SsrNode> = view! { cx, div(ref=noderef) };
        assert!(noderef.try_get::<SsrNode>().is_some());
    });
}

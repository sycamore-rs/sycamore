use super::*;

#[wasm_bindgen_test]
fn lazy() {
    let node =
        TemplateResult::new_lazy(|| TemplateResult::new_node(DomNode::text_node("Hello World!")));

    render_to(|| node, &test_container());

    assert_eq!(test_container().text_content().unwrap(), "23");
}

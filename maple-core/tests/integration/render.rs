use super::*;

#[wasm_bindgen_test]
fn lazy() {
    let node: TemplateResult<DomNode> = TemplateResult::new_lazy(|| {
        template! {
            div {
                "Test"
            }
        }
    });

    render_to(|| node, &test_container());

    assert_eq!(
        document()
            .query_selector("div")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "Test"
    );
}

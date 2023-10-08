use super::*;

#[wasm_bindgen_test]
fn issue_391_svg_with_class_should_not_use_classname() {
    sycamore::render_to(
        || {
            view! {
                svg(class="my-class")
            }
        },
        &test_container(),
    );

    let svg = query("svg");

    assert_eq!(svg.get_attribute("class").unwrap(), "my-class");
}

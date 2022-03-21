use super::*;

#[wasm_bindgen_test]
fn issue_391_svg_with_class_should_not_use_classname() {
    sycamore::render_to(
        |cx| {
            view! { cx,
                svg(class="my-class")
            }
        },
        &test_container(),
    );
}

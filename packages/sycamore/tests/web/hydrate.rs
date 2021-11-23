use web_sys::Element;

use super::*;

#[wasm_bindgen_test]
fn hello_world() {
    let html = r#"<p data-hk="0.1">Hello World!</p>"#;

    let c = test_container();
    c.clone().unchecked_into::<Element>().set_inner_html(html);

    sycamore::hydrate_to(|| view! { p { "Hello World!" } }, &c);
}

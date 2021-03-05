use web_sys::Element;

pub fn element(tag: &str) -> Element {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element(tag)
        .unwrap()
}

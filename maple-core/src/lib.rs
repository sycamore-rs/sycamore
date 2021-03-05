use web_sys::Element;

pub mod template;
pub mod internal;

pub fn start_app(element: Element) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document.body().unwrap().append_child(&element).unwrap();
}

pub mod prelude {
    pub use crate::template::Template;
    pub use crate::start_app;

    pub use maple_core_macro::template;
}

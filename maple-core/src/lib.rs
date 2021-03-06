pub mod internal;
pub mod reactive;
pub mod template;

use web_sys::HtmlElement;

pub fn render(element: impl Fn() -> HtmlElement) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document.body().unwrap().append_child(&element()).unwrap();
}

pub mod prelude {
    pub use crate::render;
    pub use crate::template::Template;

    pub use maple_core_macro::template;
}

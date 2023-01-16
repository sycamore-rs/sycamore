//! HTML and SVG tag definitions.
//!
//! _Documentation sources: <https://developer.mozilla.org/en-US/>_

use sycamore_reactive::create_scope_immediate;

pub mod elements;

mod attributes;
mod bind_props;
mod events;
mod props;

pub use attributes::attr;
pub use bind_props::bind;
pub use events::on;
pub use props::prop;

pub fn test() {
    create_scope_immediate(|cx| {
        let node = elements::button(cx)
            .with(attr::class, "bg-red-500")
            .with(on::click, |_| {})
            .into_element();
        let view = crate::View::new_node(node);

        let _ = view;
    });
}

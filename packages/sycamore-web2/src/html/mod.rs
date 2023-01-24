//! HTML and SVG tag definitions.
//!
//! _Documentation sources: <https://developer.mozilla.org/en-US/>_

use sycamore_reactive::create_scope_immediate;

pub mod elements;

mod attributes;
mod bind_props;
mod events;
mod props;

pub use attributes::{GlobalAttributes, GlobalSvgAttributes};
pub use bind_props::bind;
pub use events::{on, OnAttributes};
pub use props::prop;

pub fn test() {
    use elements::*;
    create_scope_immediate(|cx| {
        let view = button(cx).class("bg-red-500").view();

        let _ = view;
    });
}

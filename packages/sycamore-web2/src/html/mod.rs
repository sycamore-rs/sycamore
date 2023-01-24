//! HTML and SVG tag definitions.
//!
//! _Documentation sources: <https://developer.mozilla.org/en-US/>_

pub mod elements;

mod attributes;
mod bind_props;
mod events;
mod props;

pub use attributes::{GlobalAttributes, GlobalSvgAttributes};
pub use bind_props::{bind, BindAttributes};
pub use elements::*;
pub use events::{on, OnAttributes};
pub use props::{prop, PropAttributes};

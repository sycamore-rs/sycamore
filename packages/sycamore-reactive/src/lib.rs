//! Reactive primitives for [Sycamore](https://github.com/sycamore-rs/sycamore).
//!
//! ```rust
//! use sycamore_reactive::*;
//!
//! create_root(|| {
//!     let greeting = create_signal("Hello");
//!     let name = create_signal("World");
//!
//!     let display_text = create_memo(move || format!("{greeting} {name}!"));
//!     assert_eq!(display_text.get_clone(), "Hello World!");
//!
//!     name.set("Sycamore");
//!     assert_eq!(display_text.get_clone(), "Hello Sycamore!");
//! });
//! ```
//!
//! # A note on `nightly`
//!
//! If you are using rust `nightly`, you can enable the `nightly` feature to enable the more terse
//! syntax for signal get/set.
//!
//! ```ignore
//! # use sycamore_reactive::*;
//! # create_root(|| {
//! let signal = create_signal(123);
//!
//! // Stable:
//! let value = signal.get();
//! signal.set(456);
//!
//! // Nightly:
//! let value = signal();
//! signal(456);
//! # });
//! ```
//! Of course, the stable `.get()` also works on nightly as well if that's what you prefer.

#![cfg_attr(feature = "nightly", feature(fn_traits, unboxed_closures))]

mod context;
mod effects;
mod iter;
mod memos;
mod node;
mod root;
mod signals;
mod utils;

pub use context::*;
pub use effects::*;
pub use iter::*;
pub use memos::*;
pub use node::*;
pub use root::*;
pub use signals::*;
pub use utils::*;

/// Add name for proc-macro purposes.
extern crate self as sycamore_reactive;

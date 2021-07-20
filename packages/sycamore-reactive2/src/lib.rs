//! Reactive primitives for Sycamore.
//!
//! For more information, read the section on [Reactivity](https://sycamore-rs.netlify.app/docs/basics/reactivity)
//! in the Sycamore Book.

mod context;
pub mod effect;
mod iter;
pub mod scope;
pub mod signal;

pub use context::*;
pub use iter::*;

/// Utility macro for cloning all the arguments and expanding the expression.
///
/// Temporary workaround for [Rust RFC #2407](https://github.com/rust-lang/rfcs/issues/2407).
///
/// # Example
/// ```
/// use sycamore_reactive2::*;
///
/// let state = Signal::new(0);
///
/// create_effect(cloned!((state) => move || {
///    state.get();
/// }));
///
/// // state still accessible outside of the effect
/// let _ = state.get();
/// ```
#[macro_export]
macro_rules! cloned {
    (($($arg:ident),*) => $e:expr) => {{
        // clone all the args
        $( let $arg = ::std::clone::Clone::clone(&$arg); )*

        $e
    }};
}

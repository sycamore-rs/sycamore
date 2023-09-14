//! Reactive primitives for [Sycamore](https://github.com/sycamore-rs/sycamore).
//!
//! ```rust
//! use sycamore_reactive3::*;
//!
//! create_root(|cx| {
//!     let greeting = create_signal(cx, "Hello");
//!     let name = create_signal(cx, "World");
//!
//!     let display_text = create_memo(cx, move || format!("{greeting} {name}!"));
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
//! ```rust
//! # use sycamore_reactive3::*;
//! # create_root(|cx| {
//! let signal = create_signal(cx, 123);
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

#![cfg_attr(feature = "nightly", feature(fn_traits))]
#![cfg_attr(feature = "nightly", feature(unboxed_closures))]

mod context;
mod effects;
mod iter;
mod memos;
mod scope;
mod signals;
mod store;
mod utils;

pub use context::*;
pub use effects::*;
pub use iter::*;
pub use memos::*;
pub use scope::*;
pub use signals::*;
pub use store::*;
pub use utils::*;

/// Add name for proc-macro purposes.
extern crate self as sycamore_reactive3;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn cleanup() {
        create_root(|cx| {
            let cleanup_called = create_signal(cx, false);
            let scope = create_child_scope(cx, |cx| {
                on_cleanup(cx, move || {
                    cleanup_called.set(true);
                });
            });
            assert!(!cleanup_called.get());
            scope.dispose();
            assert!(cleanup_called.get());
        });
    }

    #[test]
    fn cleanup_in_effect() {
        create_root(|cx| {
            let trigger = create_signal(cx, ());

            let counter = create_signal(cx, 0);

            create_effect_scoped(cx, move |cx| {
                trigger.track();

                on_cleanup(cx, move || {
                    counter.set(counter.get() + 1);
                });
            });

            assert_eq!(counter.get(), 0);

            trigger.set(());
            assert_eq!(counter.get(), 1);

            trigger.set(());
            assert_eq!(counter.get(), 2);
        });
    }

    #[test]
    fn cleanup_is_untracked() {
        create_root(|cx| {
            let trigger = create_signal(cx, ());

            let counter = create_signal(cx, 0);

            create_effect_scoped(cx, move |cx| {
                counter.set(counter.get_untracked() + 1);

                on_cleanup(cx, move || {
                    trigger.track(); // trigger should not be tracked
                });
            });

            assert_eq!(counter.get(), 1);

            trigger.set(());
            assert_eq!(counter.get(), 1);
        });
    }

    #[test]
    fn batch_updates_effects_at_end() {
        create_root(|cx| {
            let state1 = create_signal(cx, 1);
            let state2 = create_signal(cx, 2);
            let counter = create_signal(cx, 0);
            create_effect(cx, move || {
                counter.set(counter.get_untracked() + 1);
                let _ = state1.get() + state2.get();
            });
            assert_eq!(counter.get(), 1);
            state1.set(2);
            state2.set(3);
            assert_eq!(counter.get(), 3);
            batch(cx, move || {
                state1.set(3);
                assert_eq!(counter.get(), 3);
                state2.set(4);
                assert_eq!(counter.get(), 3);
            });
            assert_eq!(counter.get(), 4);
        });
    }
}

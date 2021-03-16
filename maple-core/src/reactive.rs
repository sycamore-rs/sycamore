//! Reactive primitives.

mod effect;
mod signal;

use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

pub use effect::*;
pub use signal::*;

/// Creates a new reactive root. Generally, you won't need this method as it is called automatically in [`render`](crate::render()).
///
/// # Example
/// ```
/// use maple_core::prelude::*;
///
/// let trigger = Signal::new(());
/// let counter = Signal::new(0);
///
/// let owner = create_root(cloned!((trigger, counter) => move || {
///     create_effect(move || {
///         trigger.get(); // subscribe to trigger
///         counter.set(*counter.get_untracked() + 1);
///     });
/// }));
///
/// assert_eq!(*counter.get(), 1);
///
/// trigger.set(());
/// assert_eq!(*counter.get(), 2);
///
/// drop(owner);
/// trigger.set(());
/// assert_eq!(*counter.get(), 2); // should not be updated because owner was dropped
/// ```
#[must_use = "create_root returns the owner of the effects created inside this scope"]
pub fn create_root(callback: impl FnOnce()) -> Owner {
    OWNER.with(|owner| {
        let outer_owner = owner.replace(Some(Owner::new()));
        callback();

        owner.replace(outer_owner).unwrap()
    })
}

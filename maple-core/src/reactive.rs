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

/// Creates a new reactive root. Generally, you won't need this method as it is called automatically in [`render`](crate::render).
pub fn create_root(callback: Box<dyn Fn()>) {
    OWNERS.with(|owners| {
        owners.borrow_mut().push(Owner {});
        callback();
        owners.borrow_mut().pop().unwrap(); // destroy all effects created inside scope
    });
}

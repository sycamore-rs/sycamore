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
pub fn create_root(callback: Box<dyn FnOnce()>) -> Rc<RefCell<Owner>> {
    OWNER.with(|owner| {
        *owner.borrow_mut() = Some(Rc::new(RefCell::new(Owner::new())));
        callback();

        owner.borrow_mut().as_ref().take().unwrap().clone()
    })
}

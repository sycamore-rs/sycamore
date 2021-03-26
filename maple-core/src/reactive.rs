//! Reactive primitives.

mod effect;
mod signal;
mod signal_vec;

pub use effect::*;
pub use signal::*;
pub use signal_vec::*;

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
pub fn create_root<'a>(callback: impl FnOnce() + 'a) -> Owner {
    /// Internal implementation: use dynamic dispatch to reduce code bloat.
    fn internal<'a>(callback: Box<dyn FnOnce() + 'a>) -> Owner {
        OWNER.with(|owner| {
            let outer_owner = owner.replace(Some(Owner::new()));
            callback();

            owner
                .replace(outer_owner)
                .expect("Owner should be valid inside the reactive root")
        })
    }

    internal(Box::new(callback))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn drop_owner_inside_effect() {
        let owner = Rc::new(RefCell::new(None));

        *owner.borrow_mut() = Some(create_root({
            let owner = Rc::clone(&owner);
            move || {
                let owner = owner.take();
                drop(owner)
            }
        }));
    }
}

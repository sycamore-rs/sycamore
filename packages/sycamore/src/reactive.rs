//! Reactive primitives.

mod effect;
mod iter;
mod motion;
mod signal;

pub use effect::*;
pub use iter::*;
pub use motion::*;
pub use signal::*;

/// Creates a new reactive root / scope. Generally, you won't need this method as it is called
/// automatically in [`render`](crate::generic_node::render).
///
/// # Example
/// ```
/// use sycamore::prelude::*;
///
/// let trigger = Signal::new(());
/// let counter = Signal::new(0);
///
/// let scope = create_root(cloned!((trigger, counter) => move || {
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
/// drop(scope);
/// trigger.set(());
/// assert_eq!(*counter.get(), 2); // should not be updated because scope was dropped
/// ```
#[must_use = "create_root returns the reactive scope of the effects created inside this scope"]
pub fn create_root<'a>(callback: impl FnOnce() + 'a) -> ReactiveScope {
    /// Internal implementation: use dynamic dispatch to reduce code bloat.
    fn internal<'a>(callback: Box<dyn FnOnce() + 'a>) -> ReactiveScope {
        SCOPE.with(|scope| {
            let outer_scope = scope.replace(Some(ReactiveScope::new()));
            callback();

            scope
                .replace(outer_scope)
                .expect("ReactiveScope should be valid inside the reactive root")
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
    fn drop_scope_inside_effect() {
        let scope = Rc::new(RefCell::new(None));

        *scope.borrow_mut() = Some(create_root({
            let scope = Rc::clone(&scope);
            move || {
                let scope = scope.take();
                drop(scope);
            }
        }));
    }
}

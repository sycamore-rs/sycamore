//! Reactive primitives for Sycamore.
//!
//! To learn more, read the section on [Reactivity](https://sycamore-rs.netlify.app/docs/basics/reactivity)
//! in the Sycamore Book.

mod context;
mod effect;
mod iter;
mod signal;

pub use context::*;
pub use effect::*;
pub use iter::*;
pub use signal::*;

use wasm_bindgen::prelude::*;

/// Creates a new reactive root / scope. Generally, you won't need this method as it is called
/// automatically in `render`.
///
/// # Example
/// ```
/// use sycamore_reactive::*;
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
    _create_root(Box::new(callback))
}

/// Internal implementation: use dynamic dispatch to reduce code bloat.
fn _create_root<'a>(callback: Box<dyn FnOnce() + 'a>) -> ReactiveScope {
    SCOPES.with(|scopes| {
        // Push new empty scope on the stack.
        let scope = ReactiveScope::new();

        if let Some(parent) = scopes.borrow().last() {
            scope.0.borrow_mut().parent = parent.downgrade();
        }
        scopes.borrow_mut().push(scope);
        callback();

        // Pop the scope from the stack and return it.
        scopes.borrow_mut().pop().unwrap_throw()
    })
}

/// Utility macro for cloning all the arguments and expanding the expression.
///
/// Temporary workaround for [Rust RFC #2407](https://github.com/rust-lang/rfcs/issues/2407).
///
/// # Example
/// ```
/// use sycamore_reactive::*;
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

    #[test]
    fn cloned() {
        let state = Signal::new(0);

        let _x = cloned!((state) => state);

        // state still accessible because it was cloned instead of moved
        let _ = state.get();
    }

    #[test]
    fn cloned_closure() {
        let state = Signal::new(0);

        create_effect(cloned!((state) => move || {
            state.get();
        }));

        // state still accessible outside of the effect
        let _ = state.get();
    }
}

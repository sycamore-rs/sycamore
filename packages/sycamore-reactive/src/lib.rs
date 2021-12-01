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
/// The parent of the created [`ReactiveScope`] is automatically set to the current scope.
/// If this behavior is not intended, see [`create_child_scope_in`].
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// let trigger = Signal::new(());
/// let counter = Signal::new(0);
///
/// let scope = create_scope(cloned!((trigger, counter) => move || {
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
#[must_use = "create_scope returns the reactive scope of the effects created inside this scope"]
#[cfg_attr(debug_assertions, track_caller)]
pub fn create_scope<'a>(callback: impl FnOnce() + 'a) -> ReactiveScope {
    _create_child_scope_in(&ReactiveScopeWeak::default(), Box::new(callback))
}

/// Creates a [`ReactiveScope`] with the specified parent scope. The parent scope does not
/// necessarily need to be the current scope.
///
/// In general, prefer [`create_scope`]. This method is useful when scopes can be created outside of
/// the initial code path (e.g. inside a spawned future) and the scope hierarchy needs to be
/// conserved (e.g. to be able to access contexts).
#[must_use = "create_child_scope_in returns the reactive scope of the effects created inside this scope"]
#[cfg_attr(debug_assertions, track_caller)]
pub fn create_child_scope_in<'a>(
    parent: &ReactiveScopeWeak,
    callback: impl FnOnce() + 'a,
) -> ReactiveScope {
    _create_child_scope_in(parent, Box::new(callback))
}

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
#[deprecated(note = "use create_scope instead", since = "0.7.0")]
#[cfg_attr(debug_assertions, track_caller)]
pub fn create_root<'a>(callback: impl FnOnce() + 'a) -> ReactiveScope {
    _create_child_scope_in(&ReactiveScopeWeak::default(), Box::new(callback))
}

/// Internal implementation: use dynamic dispatch to reduce code bloat.
#[cfg_attr(debug_assertions, track_caller)]
fn _create_child_scope_in<'a>(
    parent: &ReactiveScopeWeak,
    callback: Box<dyn FnOnce() + 'a>,
) -> ReactiveScope {
    // Push new empty scope on the stack.
    // We make sure to create the ReactiveScope outside of the closure so that track_caller can do
    // its thing.
    let scope = ReactiveScope::new();

    SCOPES.with(|scopes| {
        // If `parent` was specified, use it as the parent of the new scope. Else use the parent of
        // the scope this function is called in.

        // If the ReactiveScopeWeak points to nowhere, strong_count is 0.
        if parent.0.strong_count() != 0 {
            scope.0.borrow_mut().parent = parent.clone();
        } else if let Some(current_scope) = scopes.borrow().last() {
            scope.0.borrow_mut().parent = current_scope.downgrade();
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
    ($($arg:ident),* => $e:expr) => {
        cloned!(($($arg),*) => $e)
    };
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn drop_scope_inside_effect() {
        let scope = Rc::new(RefCell::new(None));

        *scope.borrow_mut() = Some(create_scope({
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

use std::any::{Any, TypeId};
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::*;

/// Trait for any type of context.
///
/// # Equality
/// A `ContextAny` is equal to another `ContextAny` if they are of the same type.
pub(super) trait ContextAny {
    /// Get the [`TypeId`] of the type of the value stored in the context.
    fn get_type_id(&self) -> TypeId;

    /// Get the value stored in the context. The concrete type of the returned value is guaranteed
    /// to match the type when calling [`get_type_id`](ContextAny::get_type_id).
    fn get_value(&self) -> &dyn Any;

    /// Get the name of type of context or `None` if not available.
    fn get_type_name(&self) -> Option<&'static str>;
}

/// Inner representation of a context.
struct Context<T: 'static> {
    value: T,
    /// The type name of the context. Only available in debug mode.
    #[cfg(debug_assertions)]
    type_name: &'static str,
}

impl<T: 'static> ContextAny for Context<T> {
    fn get_type_id(&self) -> TypeId {
        self.value.type_id()
    }

    fn get_value(&self) -> &dyn Any {
        &self.value
    }

    fn get_type_name(&self) -> Option<&'static str> {
        #[cfg(debug_assertions)]
        return Some(self.type_name);
        #[cfg(not(debug_assertions))]
        return None;
    }
}

/// Get the value of a context in the current [`ReactiveScope`] or `None` if not found.
///
/// For a panicking version of this function, see [`use_context`].
pub fn try_use_context<T: Clone + 'static>() -> Option<T> {
    SCOPES.with(|scopes| {
        let scopes = scopes.borrow();
        let mut current = scopes.last().map(|s| Rc::clone(&s.0));
        match current {
            Some(_) => {
                while let Some(scope) = &current {
                    if let Some(context) = &scope.borrow().context {
                        if let Some(value) = context.get_value().downcast_ref::<T>() {
                            return Some(value.clone());
                        }
                    }
                    current = current.unwrap_throw().borrow().parent.0.upgrade();
                }
                None
            }
            None => None,
        }
    })
}

/// Get the value of a context in the current [`ReactiveScope`].
///
/// # Panics
/// This function will `panic!` if the context is not found in the current scope or a parent scope.
/// For a non-panicking version of this function, see [`try_use_context`].
#[track_caller]
pub fn use_context<T: Clone + 'static>() -> T {
    try_use_context().expect("context not found for type")
}

/// Creates a new [`ReactiveScope`] with a context and runs the supplied callback function.
#[cfg_attr(debug_assertions, track_caller)]
pub fn create_context_scope<T: 'static, Out>(value: T, f: impl FnOnce() -> Out) -> Out {
    // Create a new ReactiveScope.
    // We make sure to create the ReactiveScope outside of the closure so that track_caller can do
    // its thing.
    let scope = create_scope(|| {});
    SCOPES.with(|scopes| {
        // Attach the context to the scope.
        scope.0.borrow_mut().context = Some(Box::new(Context {
            value,
            #[cfg(debug_assertions)]
            type_name: std::any::type_name::<T>(),
        }));
        scopes.borrow_mut().push(scope);
        let out = f();
        let scope = scopes.borrow_mut().pop().unwrap_throw();
        on_cleanup(move || drop(scope));
        out
    })
}

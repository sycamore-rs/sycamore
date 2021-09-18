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
}

/// Inner representation of a context.
struct Context<T: 'static> {
    value: T,
}

impl<T: 'static> ContextAny for Context<T> {
    fn get_type_id(&self) -> TypeId {
        self.value.type_id()
    }

    fn get_value(&self) -> &dyn Any {
        &self.value
    }
}

/// Get the value of a context in the current [`ReactiveScope`].
///
/// # Panics
/// This function will `panic!` if the context is not found in the current scope or a parent scope.
pub fn use_context<T: Clone + 'static>() -> T {
    SCOPES.with(|scopes| {
        let scopes = scopes.borrow();
        let mut current = scopes.last().map(|s| Rc::clone(&s.0));
        match current {
            Some(_) => {
                while let Some(scope) = &current {
                    if let Some(context) = &scope.borrow().context {
                        if let Some(value) = context.get_value().downcast_ref::<T>() {
                            return value.clone();
                        }
                    }
                    current = current.unwrap_throw().borrow().parent.0.upgrade();
                }
                panic!("context not found for type")
            }
            None => panic!("context not found for type"),
        }
    })
}

/// Creates a new [`ReactiveScope`] with a context and runs the supplied callback function.
pub fn create_context_scope<T: 'static, Out>(value: T, f: impl FnOnce() -> Out) -> Out {
    SCOPES.with(|scopes| {
        // Create a new ReactiveScope with a context.
        let scope = ReactiveScope::new();
        scope.0.borrow_mut().context = Some(Box::new(Context { value }));
        scopes.borrow_mut().push(scope);
        let out = f();
        let scope = scopes.borrow_mut().pop().unwrap_throw();
        on_cleanup(move || drop(scope));
        out
    })
}

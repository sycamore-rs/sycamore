use std::any::{Any, TypeId};

use super::*;

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
        // Walk up the scope stack until we find a context that matches type or `panic!`.
        for scope in scopes.borrow().iter().rev() {
            if let Some(context) = &scope.context {
                if let Some(value) = context.get_value().downcast_ref::<T>() {
                    return value.clone();
                }
            }
        }

        panic!("context not found for type");
    })
}

use std::any::{Any, TypeId};

use crate::prelude::*;

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

/// Props for [`ContextProvider`].
pub struct ContextProviderProps<T, F, G>
where
    T: 'static,
    F: FnOnce() -> Template<G>,
    G: GenericNode,
{
    value: T,
    children: F,
}

/// Add a new context to the current [`ReactiveScope`].
#[component(ContextProvider<G>)]
pub fn context_provider<T, F>(props: ContextProviderProps<T, F, G>) -> Template<G>
where
    T: 'static,
    F: FnOnce() -> Template<G>,
{
    let ContextProviderProps { value, children } = props;

    SCOPES.with(|scopes| {
        // Add context to the current scope.
        if let Some(scope) = scopes.borrow_mut().last_mut() {
            scope
                .contexts
                .insert(value.type_id(), Box::new(Context { value }));

            children()
        } else {
            panic!("ContextProvider must be used inside ReactiveScope");
        }
    })
}

/// Get the value of a context in the current [`ReactiveScope`].
///
/// # Panics
/// This function will `panic!` if the context is not found in the current scope or a parent scope.
pub fn use_context<T: Clone + 'static>() {
    SCOPES.with(|scopes| {
        // Walk up the scope stack until we find a context that matches type or `panic!`.
        for scope in scopes.borrow().iter().rev() {
            if let Some(context) = scope.contexts.get(&TypeId::of::<T>()) {
                let value = context.get_value().downcast_ref::<T>().unwrap();
                return value.clone();
            }
        }

        panic!("context not found for type");
    });
}

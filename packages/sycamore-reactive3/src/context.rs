//! Context values.

use std::any::{type_name, Any};

use crate::Scope;

/// Provide a context value in this scope.
///
/// # Panics
/// This panics if a context value of the same type exists already in this scope. Note that it is
/// allowed to have context values with the same type in _different_ scopes.
pub fn provide_context<T: 'static>(cx: Scope, value: T) {
    let any: Box<dyn Any> = Box::new(value);
    // Check if a context of the same type exists already.
    cx.get_data(|scope| {
        if scope
            .context
            .iter()
            .any(|x| (**x).type_id() == (*any).type_id())
        {
            panic!(
                "a context with type {} exists already in this scope",
                type_name::<T>()
            );
        } else {
            scope.context.push(any);
        }
    });
}

/// Provide a context value in global scope.
///
/// # Panics
/// This panics if a context value of the same type exists already in the global scope. Note that it
/// is allowed to have context values with the same type in _different_ scopes.
pub fn provide_global_context<T: 'static>(cx: Scope, value: T) {
    let global = cx.root.root_scope.get();
    provide_context(
        Scope {
            id: global,
            root: cx.root,
        },
        value,
    );
}

/// Tries to get a context value of the given type. If no context is found, returns `None`.
pub fn try_use_context<T: Clone + 'static>(cx: Scope) -> Option<T> {
    // Walk up the scope stack until we find one with the context of the right type.
    cx.get_data(|scope| {
        for value in &scope.context {
            if let Some(value) = value.downcast_ref::<T>().cloned() {
                return Some(value);
            }
        }

        // No context of the right type found for this scope. Now check the parent scope.
        if let Some(parent) = scope.parent {
            try_use_context::<T>(Scope {
                id: parent,
                root: cx.root,
            })
        } else {
            None
        }
    })
}

/// Get a context with the given type. If no context is found, this panics.
pub fn use_context<T: Clone + 'static>(cx: Scope) -> T {
    try_use_context(cx).unwrap_or_else(|| panic!("no context of type {} found)", type_name::<T>()))
}

/// Gets how deep the current scope is from the root/global scope. The value for the global scope
/// itself is always `0`.
pub fn use_scope_depth(cx: Scope) -> u32 {
    cx.get_data(|scope| {
        if let Some(parent) = scope.parent {
            use_scope_depth(Scope {
                id: parent,
                root: cx.root,
            }) + 1
        } else {
            0
        }
    })
}

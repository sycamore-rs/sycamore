//! Context values.

use std::any::{type_name, Any};

use crate::{Root, ScopeId};

/// Provide a context value in this scope.
///
/// # Panics
/// This panics if a context value of the same type exists already in this scope. Note that it is
/// allowed to have context values with the same type in _different_ scopes.
pub fn provide_context<T: 'static>(value: T) {
    let root = Root::get_global();
    provide_context_in_scope(root.current_scope.get(), value);
}

/// Provide a context value in global scope.
///
/// # Panics
/// This panics if a context value of the same type exists already in the global scope. Note that it
/// is allowed to have context values with the same type in _different_ scopes.
pub fn provide_global_context<T: 'static>(value: T) {
    let root = Root::get_global();
    provide_context_in_scope(root.root_scope.get(), value);
}

/// Internal implementation for [`provide_context`] and [`provide_global_context`].
fn provide_context_in_scope<T: 'static>(key: ScopeId, value: T) {
    let root = Root::get_global();
    let mut scopes = root.scopes.borrow_mut();
    let any: Box<dyn Any> = Box::new(value);

    let scope = &mut scopes[key];
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
}

/// Tries to get a context value of the given type. If no context is found, returns `None`.
pub fn try_use_context<T: Clone + 'static>() -> Option<T> {
    let root = Root::get_global();
    let scopes = root.scopes.borrow();
    // Walk up the scope stack until we find one with the context of the right type.
    let mut current = Some(&scopes[root.current_scope.get()]);
    while let Some(next) = current {
        for value in &next.context {
            if let Some(value) = value.downcast_ref::<T>().cloned() {
                return Some(value);
            }
        }
        // No context of the right type found for this scope. Now check the parent scope.
        current = next.parent.map(|key| &scopes[key]);
    }
    None
}

/// Get a context with the given type. If no context is found, this panics.
pub fn use_context<T: Clone + 'static>() -> T {
    try_use_context().unwrap_or_else(|| panic!("no context of type {} found)", type_name::<T>()))
}

/// Gets how deep the current scope is from the root/global scope. The value for the global scope
/// itself is always `0`.
pub fn use_scope_depth() -> u32 {
    let root = Root::get_global();
    let scopes = root.scopes.borrow();
    let mut current = Some(&scopes[root.current_scope.get()]);
    let mut depth = 0;

    while let Some(next) = current {
        current = next.parent.map(|key| &scopes[key]);
        depth += 1;
    }
    depth
}

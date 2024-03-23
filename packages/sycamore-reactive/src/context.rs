//! Context values.

use std::any::{type_name, Any};

use slotmap::Key;

use crate::{NodeId, Root};

/// Provide a context value in this scope.
///
/// # Panics
/// This panics if a context value of the same type exists already in this scope. Note that it is
/// allowed to have context values with the same type in _different_ scopes.
#[cfg_attr(debug_assertions, track_caller)]
pub fn provide_context<T: 'static>(value: T) {
    let root = Root::global();
    provide_context_in_node(root.current_node.get(), value);
}

/// Internal implementation for [`provide_context`] and [`provide_global_context`].
#[cfg_attr(debug_assertions, track_caller)]
fn provide_context_in_node<T: 'static>(id: NodeId, value: T) {
    let root = Root::global();
    let mut nodes = root.nodes.borrow_mut();
    let any: Box<dyn Any> = Box::new(value);

    let node = &mut nodes[id];
    if node
        .context
        .iter()
        .any(|x| (**x).type_id() == (*any).type_id())
    {
        panic!(
            "a context with type `{}` exists already in this scope",
            type_name::<T>()
        );
    }
    node.context.push(any);
}

/// Tries to get a context value of the given type. If no context is found, returns `None`.
pub fn try_use_context<T: Clone + 'static>() -> Option<T> {
    let root = Root::global();
    let nodes = root.nodes.borrow();
    // Walk up the scope stack until we find one with the context of the right type.
    let mut current = Some(&nodes[root.current_node.get()]);
    while let Some(next) = current {
        for value in &next.context {
            if let Some(value) = value.downcast_ref::<T>().cloned() {
                return Some(value);
            }
        }
        // No context of the right type found for this scope. Now check the parent scope.
        if next.parent.is_null() {
            current = None;
        } else {
            current = Some(&nodes[next.parent]);
        }
    }
    None
}

/// Get a context with the given type. If no context is found, this panics.
#[cfg_attr(debug_assertions, track_caller)]
pub fn use_context<T: Clone + 'static>() -> T {
    if let Some(value) = try_use_context() {
        value
    } else {
        panic!("no context of type `{}` found", type_name::<T>())
    }
}

/// Try to get a context with the given type. If no context is found, returns the value of the
/// function and sets the value of the context in the current scope.
pub fn use_context_or_else<T: Clone + 'static, F: FnOnce() -> T>(f: F) -> T {
    try_use_context().unwrap_or_else(|| {
        let value = f();
        provide_context(value.clone());
        value
    })
}

/// Gets how deep the current scope is from the root/global scope. The value for the global scope
/// itself is always `0`.
pub fn use_scope_depth() -> u32 {
    let root = Root::global();
    let nodes = root.nodes.borrow();
    let mut current = Some(&nodes[root.current_node.get()]);
    let mut depth = 0;

    while let Some(next) = current {
        depth += 1;
        if next.parent.is_null() {
            current = None;
        } else {
            current = Some(&nodes[next.parent]);
        }
    }
    depth
}

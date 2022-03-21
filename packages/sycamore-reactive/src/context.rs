//! Context state management.

use crate::*;

/// Provides a context in the current [`Scope`]. The context can later be accessed by using
/// [`use_context`] lower in the scope hierarchy.
///
/// The context can also be accessed in the same scope in which it is provided.
///
/// This method is simply a wrapper around [`create_ref`] and [`provide_context_ref`].
///
/// # Panics
/// This method panics if a context with the same type exists already in this scope.
/// Note that if a context with the same type exists in a parent scope, the new context will
/// shadow the old context.
#[track_caller]
pub fn provide_context<T: 'static>(cx: Scope, value: T) -> &T {
    let value = create_ref(cx, value);
    provide_context_ref(cx, value)
}

/// Provides a context in the current [`Scope`]. The context can later be accessed by using
/// [`use_context`] lower in the scope hierarchy.
///
/// The context can also be accessed in the same scope in which it is provided.
///
/// Unlike [`provide_context`], this method accepts a reference that
/// lives at least as long as the scope.
///
/// # Panics
/// This method panics if a context with the same type exists already in this scope.
/// Note that if a context with the same type exists in a parent scope, the new context will
/// shadow the old context.
#[track_caller]
pub fn provide_context_ref<'a, T: 'static>(cx: Scope<'a>, value: &'a T) -> &'a T {
    let type_id = TypeId::of::<T>();
    if cx
        .raw
        .inner
        .borrow_mut()
        .contexts
        .get_or_insert_with(Default::default)
        .insert(type_id, value)
        .is_some()
    {
        panic!("existing context with type exists already");
    }
    value
}

/// Tries to get a context value of the given type. If no context with the right type found,
/// returns `None`. For a panicking version, see [`use_context`].
pub fn try_use_context<T: 'static>(cx: Scope) -> Option<&T> {
    let type_id = TypeId::of::<T>();
    let mut this = Some(cx.raw);
    while let Some(current) = this {
        if let Some(value) = current
            .inner
            .borrow()
            .contexts
            .as_ref()
            .and_then(|c| c.get(&type_id))
        {
            let value = value.downcast_ref::<T>().unwrap();
            return Some(value);
        } else {
            // SAFETY: `current.parent` necessarily lives longer than `current`.
            this = current.parent.map(|x| unsafe { &*x });
        }
    }
    None
}

/// Gets a context value of the given type.
///
/// # Panics
/// This method panics if the context cannot be found in the current scope hierarchy.
/// For a non-panicking version, see [`try_use_context`].
#[track_caller]
pub fn use_context<T: 'static>(cx: Scope) -> &T {
    try_use_context(cx).expect("context not found for type")
}

/// Gets a context value of the given type or computes it from a closure.
pub fn use_context_or_else<T: 'static>(cx: Scope, f: impl FnOnce() -> T) -> &T {
    try_use_context(cx).unwrap_or_else(|| provide_context(cx, f()))
}

/// Returns the current depth of the scope. If the scope is the root scope, returns `0`.
pub fn scope_depth(cx: Scope) -> u32 {
    let mut depth = 0;
    let mut this = Some(cx.raw);
    while let Some(current) = this {
        // SAFETY: `current.parent` necessarily lives longer than `current`.
        this = current.parent.map(|x| unsafe { &*x });
        depth += 1;
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context() {
        create_scope_immediate(|cx| {
            provide_context(cx, 42i32);
            let x = use_context::<i32>(cx);
            assert_eq!(*x, 42);
        });
    }

    #[test]
    fn context_in_nested_scope() {
        create_scope_immediate(|cx| {
            provide_context(cx, 42i32);
            let _ = create_child_scope(cx, |cx| {
                let x = use_context::<i32>(cx);
                assert_eq!(*x, 42);
            });
        });
    }

    #[test]
    // Do not run under miri as there is a memory leak false positive.
    #[cfg_attr(miri, ignore)]
    #[should_panic = "existing context with type exists already"]
    fn existing_context_with_same_type_should_panic() {
        create_scope_immediate(|cx| {
            provide_context(cx, 0i32);
            provide_context(cx, 0i32);
            //                  ^^^^ -> has type `i32` and therefore should panic
        });
    }
}

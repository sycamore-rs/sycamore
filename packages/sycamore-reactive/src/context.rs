//! Context state management.

use crate::*;

impl<'a> Scope<'a> {
    /// Provides a context in the current [`Scope`]. The context can later be accessed by using
    /// [`use_context`](Self::use_context) lower in the scope hierarchy.
    ///
    /// The context can also be accessed in the same scope in which it is provided.
    ///
    /// # Panics
    /// This method panics if a context with the same type exists already in this scope.
    /// Note that if a context with the same type exists in a parent scope, the new context will
    /// shadow the old context.
    pub fn provide_context<T: 'static>(&'a self, value: T) {
        let type_id = TypeId::of::<T>();
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        if self.contexts.borrow_mut().insert(type_id, ptr).is_some() {
            panic!("existing context with type exists already");
        }
    }

    /// Tries to get a context value of the given type. If no context with the right type found,
    /// returns `None`. For a panicking version, see [`use_context`](Self::use_context).
    pub fn try_use_context<T: 'static>(&'a self) -> Option<&'a T> {
        let type_id = TypeId::of::<T>();
        let mut this = Some(self);
        while let Some(current) = this {
            if let Some(value) = current.contexts.borrow_mut().get(&type_id) {
                // SAFETY: value lives at least as long as 'a:
                // - Lifetime of value is 'a if it is allocated on the current scope.
                // - Lifetime of value is longer than 'a if it is allocated on a parent scope.
                // - 'a is variant because it is an immutable reference.
                let value = unsafe { &**value };
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
    /// For a non-panicking version, see [`try_use_context`](Self::try_use_context).
    #[track_caller]
    pub fn use_context<T: 'static>(&'a self) -> &'a T {
        self.try_use_context().expect("context not found for type")
    }

    /// Returns the current depth of the scope. If the scope is the root scope, returns `0`.
    pub fn scope_depth(&self) -> u32 {
        let mut depth = 0;
        let mut this = Some(self);
        while let Some(current) = this {
            // SAFETY: `current.parent` necessarily lives longer than `current`.
            this = current.parent.map(|x| unsafe { &*x });
            depth += 1;
        }
        depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context() {
        create_scope_immediate(|ctx| {
            ctx.provide_context(42i32);
            let x = ctx.use_context::<i32>();
            assert_eq!(*x, 42);
        });
    }

    #[test]
    fn context_in_nested_scope() {
        create_scope_immediate(|ctx| {
            ctx.provide_context(42i32);
            let _ = ctx.create_child_scope(|ctx| {
                let x = ctx.use_context::<i32>();
                assert_eq!(*x, 42);
            });
        });
    }
}

//! Context state management.

use crate::*;

impl<'a> Scope<'a> {
    /// Provides a context in the current [`Scope`]. The context can later be accessed by using
    /// [`use_context`](Self::use_context) lower in the scope hierarchy.
    ///
    /// The context can also be accessed in the same scope in which it is provided.
    ///
    /// This method is simply a wrapper around [`create_ref`](Self::create_ref) and
    /// [`provide_context_ref`](Self::provide_context_ref).
    ///
    /// # Panics
    /// This method panics if a context with the same type exists already in this scope.
    /// Note that if a context with the same type exists in a parent scope, the new context will
    /// shadow the old context.
    #[track_caller]
    pub fn provide_context<T: 'static>(self, value: T) -> &'a T {
        let value = self.create_ref(value);
        self.provide_context_ref(value)
    }

    /// Provides a context in the current [`Scope`]. The context can later be accessed by using
    /// [`use_context`](Self::use_context) lower in the scope hierarchy.
    ///
    /// The context can also be accessed in the same scope in which it is provided.
    ///
    /// Unlike [`provide_context`](Self::provide_context), this method accepts a reference that
    /// lives at least as long as the scope.
    ///
    /// # Panics
    /// This method panics if a context with the same type exists already in this scope.
    /// Note that if a context with the same type exists in a parent scope, the new context will
    /// shadow the old context.
    #[track_caller]
    pub fn provide_context_ref<T: 'static>(self, value: &'a T) -> &'a T {
        let type_id = TypeId::of::<T>();
        if self
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
    /// returns `None`. For a panicking version, see [`use_context`](Self::use_context).
    pub fn try_use_context<T: 'static>(self) -> Option<&'a T> {
        let type_id = TypeId::of::<T>();
        let mut this = Some(self.raw);
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
    /// For a non-panicking version, see [`try_use_context`](Self::try_use_context).
    #[track_caller]
    pub fn use_context<T: 'static>(self) -> &'a T {
        self.try_use_context().expect("context not found for type")
    }

    /// Gets a context value of the given type or computes it from a closure.
    pub fn use_context_or_else<T: 'static>(self, f: impl FnOnce() -> T) -> &'a T {
        self.try_use_context()
            .unwrap_or_else(|| self.provide_context(f()))
    }

    /// Returns the current depth of the scope. If the scope is the root scope, returns `0`.
    pub fn scope_depth(&self) -> u32 {
        let mut depth = 0;
        let mut this = Some(self.raw);
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

    #[test]
    // Do not run under miri as there is a memory leak false positive.
    #[cfg_attr(miri, ignore)]
    #[should_panic = "existing context with type exists already"]
    fn existing_context_with_same_type_should_panic() {
        create_scope_immediate(|ctx| {
            ctx.provide_context(0i32);
            ctx.provide_context(0i32);
            //                  ^^^^ -> has type `i32` and therefore should panic
        });
    }
}

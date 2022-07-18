//! Arena allocator for [`Scope`](crate::Scope).

use std::cell::UnsafeCell;

use bumpalo::Bump;
use smallvec::SmallVec;

/// The size of the [`SmallVec`] inline data.
const SCOPE_ARENA_STACK_SIZE: usize = 4;

/// A trait that is implemented for everything.
pub(crate) trait ReallyAny {}
impl<T> ReallyAny for T {}

#[derive(Default)]
pub(crate) struct ScopeArena<'a> {
    bump: Bump,
    // We need to store the raw pointers because otherwise the values won't be dropped.
    inner: UnsafeCell<SmallVec<[*mut (dyn ReallyAny + 'a); SCOPE_ARENA_STACK_SIZE]>>,
}

impl<'a> ScopeArena<'a> {
    /// Allocate a value onto the arena. Returns a mutable reference that lasts as long as the arena
    /// itself.
    #[allow(clippy::mut_from_ref)] // We return a new reference each time so this is a false-positive.
    pub fn alloc<T: 'a>(&'a self, value: T) -> &'a mut T {
        let boxed = bumpalo::boxed::Box::new_in(value, &self.bump);
        let ptr = bumpalo::boxed::Box::into_raw(boxed);
        unsafe {
            // SAFETY: The only place where self.inner.get() is mutably borrowed is right here.
            // It is impossible to have two alloc() calls on the same ScopeArena at the same time so
            // the mutable reference here is effectively unique.
            let inner_exclusive = &mut *self.inner.get();
            inner_exclusive.push(ptr);
        };

        // SAFETY: the address of the ptr lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.inner is append only. That means that the Box<_> will not be dropped until Self is
        //   dropped.
        // - The drop code for Scope never reads the allocated value and therefore does not create a
        //   stacked-borrows violation.
        unsafe { &mut *ptr }
    }

    /// Cleanup the resources owned by the [`ScopeArena`]. This is automatically called in [`Drop`].
    /// However, [`dispose`](Self::dispose) only needs to take `&self` instead of `&mut self`.
    /// Dropping a [`ScopeArena`] will automatically call [`dispose`](Self::dispose).
    ///
    /// If a [`ScopeArena`] has already been disposed, calling it again does nothing.
    pub unsafe fn dispose(&self) {
        for &ptr in (*self.inner.get()).iter().rev() {
            // SAFETY: the ptr was allocated in Self::alloc using bumpalo::boxed::Box::into_raw.
            let boxed: bumpalo::boxed::Box<dyn ReallyAny> = bumpalo::boxed::Box::from_raw(ptr);
            // Call the drop code for the allocated value.
            drop(boxed);
        }
        // Clear the inner Vec to prevent dangling references.
        drop(std::mem::take(&mut *self.inner.get()));
    }
}

impl<'a> Drop for ScopeArena<'a> {
    fn drop(&mut self) {
        unsafe { self.dispose() }
    }
}

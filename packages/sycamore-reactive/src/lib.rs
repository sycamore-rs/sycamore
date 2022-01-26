//! Reactive primitives for Sycamore.

#![warn(missing_docs)]

mod arena;
mod context;
mod effect;
mod iter;
mod memo;
mod signal;

pub use effect::*;
pub use signal::*;

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use arena::*;
use indexmap::IndexMap;
use slotmap::{DefaultKey, SlotMap};

/// A wrapper type around a lifetime that forces the lifetime to be invariant.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct InvariantLifetime<'id>(PhantomData<&'id mut &'id ()>);

/// A reactive scope.
///
/// The only way to ever use a scope should be behind a reference.
/// It should never be possible to access a raw [`Scope`] on the stack.
///
/// The intended way to access a [`Scope`] is with the [`create_scope`] function.
///
/// For convenience, the [`ScopeRef`] type alias is defined as a reference to a [`Scope`].
///
/// # Lifetime
///
/// * `'a` - The lifetime of the scope and all data allocated on it. This allows passing in data
///   from an outer scope into an inner scope. This lifetime is also invariant because it is used
///   within an cell.
pub struct Scope<'a> {
    /// Effect functions created on the [`Scope`].
    effects: RefCell<Vec<Rc<RefCell<Option<EffectState<'a>>>>>>,
    /// Cleanup functions.
    cleanups: RefCell<Vec<Box<dyn FnOnce() + 'a>>>,
    /// Child scopes.
    ///
    /// The raw pointer is owned by this field.
    child_scopes: RefCell<SlotMap<DefaultKey, *mut Scope<'a>>>,
    /// An arena allocator for allocating refs and signals.
    arena: ScopeArena<'a>,
    /// Contexts that are allocated on the current [`Scope`].
    /// See the [`mod@context`] module.
    ///
    /// The raw pointer is owned by this field.
    contexts: RefCell<HashMap<TypeId, *mut (dyn Any)>>,
    /// A pointer to the parent scope.
    /// # Safety
    /// The parent scope does not actually have the right lifetime.
    parent: Option<*const Scope<'a>>,
    // Make sure that 'a is invariant.
    _phantom: InvariantLifetime<'a>,
}

impl<'a> Scope<'a> {
    /// Create a new [`Scope`]. This function is deliberately not `pub` because it should not be
    /// possible to access a [`Scope`] directly on the stack.
    pub(crate) fn new() -> Self {
        // Even though the initialization code below is same as deriving Default::default(), we
        // can't do that because accessing a raw Scope outside of a scope closure breaks
        // safety contracts.
        //
        // Self::new() is intentionally pub(crate) only to prevent end-users from creating a Scope.
        Self {
            effects: Default::default(),
            cleanups: Default::default(),
            child_scopes: Default::default(),
            arena: Default::default(),
            contexts: Default::default(),
            parent: None,
            _phantom: Default::default(),
        }
    }
}

/// A reference to a [`Scope`].
pub type ScopeRef<'a> = &'a Scope<'a>;

/// A [`ScopeRef`] that is bounded by the `'bound` lifetime. This is used to bypass restrictions on
/// HRTBs (Higher Ranked Trait Bounds) so that `'a` can be higher ranked while still be bounded by
/// the `'bound` lifetime.
pub struct BoundedScopeRef<'a, 'bound: 'a>(ScopeRef<'a>, PhantomData<&'bound ()>);

impl<'a, 'bound> BoundedScopeRef<'a, 'bound> {
    /// Create a new [`BoundedScopeRef`] from a [`ScopeRef`].
    pub fn new(ctx: ScopeRef<'a>) -> Self {
        Self(ctx, PhantomData)
    }
}

impl<'a, 'bound> Deref for BoundedScopeRef<'a, 'bound> {
    type Target = ScopeRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Creates a reactive scope.
///
/// Returns a disposer function which will release the memory owned by the [`Scope`].
/// Failure to call the disposer function will result in a memory leak.
///
/// The callback closure is called in an [untracked](untrack) scope.
///
/// # Scope lifetime
///
/// The lifetime of the child scope is arbitrary. As such, it is impossible for anything allocated
/// in the scope to escape out of the scope because it is possible for the scope lifetime to be
/// longer than outside.
///
/// ```compile_fail
/// # use sycamore_reactive::*;
/// let mut outer = None;
/// # let disposer =
/// create_scope(|ctx| {
///     outer = Some(ctx);
/// });
/// # disposer();
/// ```
///
/// # Examples
///
/// ```
/// # use sycamore_reactive::*;
/// let disposer = create_scope(|ctx| {
///     // Use ctx here.
/// });
/// disposer();
/// ```
#[must_use = "not calling the disposer function will result in a memory leak"]
pub fn create_scope(f: impl for<'a> FnOnce(ScopeRef<'a>)) -> impl FnOnce() {
    let ctx = Scope::new();
    let boxed = Box::new(ctx);
    let ptr = Box::into_raw(boxed);
    // SAFETY: Safe because heap allocated value has stable address.
    // The reference passed to f cannot possible escape the closure. We know however, that ptr
    // necessary outlives the closure call because it is only dropped in the returned disposer
    // closure.
    untrack(|| f(unsafe { &*ptr }));
    //                      ^^^ -> `ptr` is still accessible here after the call to f.

    // Ownership of `ptr` is passed into the closure.
    move || unsafe {
        // SAFETY: Safe because ptr created using Box::into_raw.
        let boxed = Box::from_raw(ptr);
        // SAFETY: Outside of call to f.
        boxed.dispose();
    }
}

/// Creates a reactive scope, runs the callback, and disposes the scope immediately.
///
/// Calling this is equivalent to writing:
/// ```
/// # use sycamore_reactive::*;
/// (create_scope(|ctx| {
///     // ...
/// }))(); // Call the disposer function immediately
/// ```
pub fn create_scope_immediate(f: impl for<'a> FnOnce(ScopeRef<'a>)) {
    let disposer = create_scope(f);
    disposer();
}

impl<'a> Scope<'a> {
    /// Create a new [`Signal`] under the current [`Scope`].
    /// The created signal lasts as long as the scope and cannot be used outside of the scope.
    ///
    /// # Signal lifetime
    ///
    /// The lifetime of the returned signal is the same as the [`Scope`].
    /// As such, the signal cannot escape the [`Scope`].
    ///
    /// ```compile_fail
    /// # use sycamore_reactive::*;
    /// let mut outer = None;
    /// create_scope_immediate(|ctx| {
    ///     let signal = ctx.create_signal(0);
    ///     outer = Some(signal);
    /// });
    /// ```
    pub fn create_signal<T>(&'a self, value: T) -> &'a Signal<T> {
        let signal = Signal::new(value);
        self.arena.alloc(signal)
    }

    /// Allocate a new arbitrary value under the current [`Scope`].
    /// The allocated value lasts as long as the scope and cannot be used outside of the scope.
    ///
    /// # Ref lifetime
    ///
    /// The lifetime of the returned ref is the same as the [`Scope`].
    /// As such, the reference cannot escape the [`Scope`].
    /// ```compile_fail
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let mut outer = None;
    /// let disposer = ctx.create_child_scope(|ctx| {
    ///     let data = ctx.create_ref(0);
    ///     let raw: &i32 = &data;
    ///     outer = Some(raw);
    ///     //           ^^^
    /// });
    /// disposer();
    /// let _ = outer.unwrap();
    /// # });
    /// ```
    pub fn create_ref<T: 'a>(&'a self, value: T) -> &'a T {
        self.arena.alloc(value)
    }

    /// Adds a callback that is called when the scope is destroyed.
    pub fn on_cleanup(&self, f: impl FnOnce() + 'a) {
        self.cleanups.borrow_mut().push(Box::new(f));
    }

    /// Create a child scope.
    ///
    /// Returns a disposer function which will release the memory owned by the [`Scope`]. If the
    /// disposer function is never called, the child scope will be disposed automatically when the
    /// parent scope is disposed.
    ///
    /// # Child scope lifetime
    ///
    /// The lifetime of the child scope is strictly a subset of the lifetime of the parent scope.
    /// ```txt
    /// [------------'a-------------]
    ///      [---------'b--------]
    /// 'a: lifetime of parent
    /// 'b: lifetime of child
    /// ```
    /// If the disposer is never called, the lifetime `'b` lasts as long as `'a`.
    /// As such, it is impossible for anything allocated in the child scope to escape into the
    /// parent scope.
    /// ```compile_fail
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let mut outer = None;
    /// let disposer = ctx.create_child_scope(|ctx| {
    ///     outer = Some(ctx);
    ///     //           ^^^
    /// });
    /// disposer();
    /// let _ = outer.unwrap();
    /// # });
    /// ```
    /// However, the closure itself only needs to live as long as the call to this method because it
    /// is called immediately. For example, the following compiles and is perfectly safe:
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let mut outer = String::new();
    /// let disposer = ctx.create_child_scope(|ctx| {
    ///     // outer is accessible inside the closure.
    ///     outer = "Hello World!".to_string();
    /// });
    /// disposer();
    /// drop(outer);
    /// //   ^^^^^ -> and remains accessible outside the closure.
    /// # });
    /// ```
    pub fn create_child_scope<F>(&'a self, f: F) -> impl FnOnce() + 'a
    where
        F: for<'child_lifetime> FnOnce(BoundedScopeRef<'child_lifetime, 'a>),
    {
        let mut child: Scope = Scope::new();
        // SAFETY: The only fields that are accessed on self from child is `context` which does not
        // have any lifetime annotations.
        child.parent = Some(unsafe { std::mem::transmute(self as *const _) });
        let boxed = Box::new(child);
        let ptr = Box::into_raw(boxed);

        let key = self
            .child_scopes
            .borrow_mut()
            // SAFETY: None of the fields of ptr are accessed through child_scopes therefore we can
            // safely transmute the lifetime.
            .insert(unsafe { std::mem::transmute(ptr) });

        // SAFETY: the address of the Ctx lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.child_ctx is append only. That means that the Box<Ctx> will not be dropped until
        //   Self is dropped.
        f(BoundedScopeRef::new(unsafe { &*ptr }));
        //                                    ^^^ -> `ptr` is still accessible here after
        // the call to f.
        move || unsafe {
            let ctx = self.child_scopes.borrow_mut().remove(key).unwrap();
            // SAFETY: Safe because ptr created using Box::into_raw and closure cannot live longer
            // than 'a.
            let ctx = Box::from_raw(ctx);
            // SAFETY: Outside of call to f.
            ctx.dispose();
        }
    }

    /// Cleanup the resources owned by the [`Scope`]. This is automatically called in [`Drop`]
    /// However, [`dispose`](Self::dispose) only needs to take `&self` instead of `&mut self`.
    /// Dropping a [`Scope`] will automatically call [`dispose`](Self::dispose).
    ///
    /// This method will cleanup resources in a certain order such that it is impossible to access a
    /// dangling-reference within cleanup callbacks and effects etc...
    ///
    /// If a [`Scope`] has already been disposed, calling it again does nothing.
    ///
    /// # Safety
    ///
    /// `dispose` should not be called inside the `create_scope` or `create_child_scope` closure.
    ///
    /// # Drop order
    ///
    /// Fields are dropped in the following order:
    /// * `child_scopes` - Run child scope drop first.
    /// * `effects`
    /// * `cleanups`
    /// * `contexts` - Contexts can be refereed to inside a cleanup callback so they are dropped
    ///   after cleanups.
    /// * `arena` - Signals and refs are dropped last because they can be refereed to in the other
    ///   fields (e.g. inside a cleanup callback).
    pub(crate) unsafe fn dispose(&self) {
        // Drop child contexts.
        for &i in self.child_scopes.take().values() {
            // SAFETY: These pointers were allocated in Self::create_child_scope.
            let ctx = Box::from_raw(i);
            // Dispose of ctx if it has not already been disposed.
            ctx.dispose()
        }
        // Drop effects.
        drop(self.effects.take());
        // Call cleanup functions in an untracked scope.
        untrack(|| {
            for cb in self.cleanups.take() {
                cb();
            }
        });
        // Cleanup context values.
        for &i in self.contexts.take().values() {
            // SAFETY: These pointers were allocated in Self::provide_context.
            drop(Box::from_raw(i));
        }
        // Cleanup signals and refs allocated on the arena.
        self.arena.dispose();
    }

    /// Returns a [`RcSignal`] that is `true` when the scope is still valid and `false` once it is
    /// disposed.
    pub fn use_scope_status(&self) -> RcSignal<bool> {
        let status = create_rc_signal(true);
        self.on_cleanup({
            let status = status.clone();
            move || status.set(false)
        });
        status
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        // SAFETY: scope cannot be dropped while it is borrowed inside closure.
        unsafe { self.dispose() };
    }
}

/// A helper function for making it explicit to define dependencies for an effect.
///
/// # Params
/// * `dependencies` - A list of [`ReadSignal`]s that are tracked.
/// * `f` - The callback function.
///
/// # Example
/// ```
/// # use sycamore_reactive::*;
/// # create_scope_immediate(|ctx| {
/// let state = ctx.create_signal(0);
///
/// ctx.create_effect(on([state], || {
///     println!("State changed. New state value = {}", state.get());
/// })); // Prints "State changed. New state value = 0"
///
/// state.set(1); // Prints "State changed. New state value = 1"
/// # });
/// ```
pub fn on<'a, const N: usize>(
    dependencies: [&'a (dyn AnyReadSignal<'a> + 'a); N],
    mut f: impl FnMut() + 'a,
) -> impl FnMut() + 'a {
    move || {
        for i in dependencies {
            i.track();
        }
        #[allow(clippy::redundant_closure)] // Clippy false-positive
        untrack(|| f())
    }
}

#[cfg(test)]
mod tests {
    use crate::{create_scope, create_scope_immediate};

    #[test]
    fn refs() {
        let disposer = create_scope(|ctx| {
            let r = ctx.create_ref(0);
            ctx.on_cleanup(move || {
                let _ = r; // r can be accessed inside scope here.
                dbg!(r);
            })
        });
        disposer();
    }

    #[test]
    fn cleanup() {
        create_scope_immediate(|ctx| {
            let cleanup_called = ctx.create_signal(false);
            let disposer = ctx.create_child_scope(|ctx| {
                ctx.on_cleanup(|| {
                    cleanup_called.set(true);
                });
            });
            assert!(!*cleanup_called.get());
            disposer();
            assert!(*cleanup_called.get());
        });
    }

    #[test]
    fn cleanup_in_effect() {
        create_scope_immediate(|ctx| {
            let trigger = ctx.create_signal(());

            let counter = ctx.create_signal(0);

            ctx.create_effect_scoped(|ctx| {
                trigger.track();

                ctx.on_cleanup(|| {
                    counter.set(*counter.get() + 1);
                });
            });

            assert_eq!(*counter.get(), 0);

            trigger.set(());
            assert_eq!(*counter.get(), 1);

            trigger.set(());
            assert_eq!(*counter.get(), 2);
        });
    }

    #[test]
    fn cleanup_is_untracked() {
        create_scope_immediate(|ctx| {
            let trigger = ctx.create_signal(());

            let counter = ctx.create_signal(0);

            ctx.create_effect_scoped(|ctx| {
                counter.set(*counter.get_untracked() + 1);

                ctx.on_cleanup(|| {
                    trigger.track(); // trigger should not be tracked
                });
            });

            assert_eq!(*counter.get(), 1);

            trigger.set(());
            assert_eq!(*counter.get(), 1);
        });
    }

    #[test]
    fn can_store_disposer_in_own_signal() {
        create_scope_immediate(|ctx| {
            let signal = ctx.create_signal(None);
            let disposer = ctx.create_child_scope(|_ctx| {});
            signal.set(Some(disposer));
        });
    }
}

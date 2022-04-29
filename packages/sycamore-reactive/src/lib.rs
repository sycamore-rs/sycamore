//! Reactive primitives for Sycamore.

#![warn(missing_docs)]

mod arena;
mod context;
mod effect;
mod iter;
mod memo;
mod signal;

pub use context::*;
pub use effect::*;
pub use iter::*;
pub use memo::*;
pub use signal::*;

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::rc::{Rc, Weak};

use arena::*;
use indexmap::IndexMap;
use slotmap::{DefaultKey, SlotMap};

/// A wrapper type around a lifetime that forces the lifetime to be invariant.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct InvariantLifetime<'id>(PhantomData<&'id mut &'id ()>);

/// Internal representation for [`Scope`]. This allows only using a single top-level [`RefCell`]
/// instead of a [`RefCell`] for every field.
#[derive(Default)]
struct ScopeInner<'a> {
    /// Effect functions created on the [`Scope`].
    effects: Vec<Rc<RefCell<Option<EffectState<'a>>>>>,
    /// Cleanup functions.
    cleanups: Vec<Box<dyn FnOnce() + 'a>>,
    /// Child scopes.
    ///
    /// The raw pointer is owned by this field.
    child_scopes: SlotMap<DefaultKey, *mut ScopeRaw<'a>>,
    /// Contexts that are allocated on the current [`Scope`].
    /// See the [`mod@context`] module.
    ///
    /// Note that the `HashMap` is wrapped with an `Option<Box<_>>`. This is because contexts are
    /// usually read and rarely created. Making this heap allocated when prevent blowing up the
    /// size of the [`ScopeInner`] struct when most of the times, this field is unneeded.
    #[allow(clippy::box_collection)]
    contexts: Option<Box<HashMap<TypeId, &'a dyn Any>>>,
    // Make sure that 'a is invariant.
    _phantom: InvariantLifetime<'a>,
}

/// What the [`BoundedScope`] points to.
struct ScopeRaw<'a> {
    inner: RefCell<ScopeInner<'a>>,
    /// An arena allocator for allocating refs and signals.
    arena: ScopeArena<'a>,
    /// A pointer to the parent scope.
    /// # Safety
    /// The parent scope does not actually have the right lifetime.
    parent: Option<*const ScopeRaw<'a>>,
}

/// A reference to a reactive scope. This reference is `Copy`, allowing it to be copied into
/// closures without any clones.
///
/// The intended way to access a [`Scope`] is with the [`create_scope`] function.
///
/// # Lifetime
///
/// * `'a` - The lifetime of the scope and all data allocated on it. This allows passing in data
///   from an outer scope into an inner scope. This lifetime is invariant because it is used within
///   an cell.
/// * `'b` - The bounded lifetime of the scope. This ensures that the scope cannot live longer than
///   this lifetime. This lifetime is covariant because if the scope can outlive `'b1`, it can also
///   outlive `'b2` if `'b1: 'b2`.
///
/// As a convenience, the [`Scope`] type alias is provided that uses the same lifetime for both `'a`
/// and `'b`. Any [`BoundedScope`] can be casted to a [`Scope`] because the second lifetime
/// parameter is always longer than the first.
#[derive(Clone, Copy)]
pub struct BoundedScope<'a, 'b: 'a> {
    raw: &'a ScopeRaw<'a>,
    /// `&'b` for covariance!
    _phantom: PhantomData<&'b ()>,
}

impl<'a, 'b: 'a> BoundedScope<'a, 'b> {
    fn new(raw: &'a ScopeRaw<'a>) -> Self {
        Self {
            raw,
            _phantom: PhantomData,
        }
    }

    /// Alias for `self.raw.arena.alloc`.
    fn alloc<T>(&self, value: T) -> &'a mut T {
        self.raw.arena.alloc(value)
    }
}

/// A type-alias for [`BoundedScope`] where both lifetimes are the same.
pub type Scope<'a> = BoundedScope<'a, 'a>;

impl<'a> ScopeRaw<'a> {
    /// Create a new [`ScopeRaw`]. This function is deliberately not `pub` because it should not be
    /// possible to access a [`ScopeRaw`] directly on the stack.
    pub(crate) fn new() -> Self {
        // Even though the initialization code below is same as deriving Default::default(), we
        // can't do that because accessing a raw Scope outside of a scope closure breaks
        // safety contracts.
        //
        // Self::new() is intentionally pub(crate) only to prevent end-users from creating a Scope.
        Self {
            inner: RefCell::new(ScopeInner {
                effects: Default::default(),
                cleanups: Default::default(),
                child_scopes: Default::default(),
                contexts: None,
                _phantom: Default::default(),
            }),
            arena: Default::default(),
            parent: None,
        }
    }
}

/// A handle that allows cleaning up a [`Scope`].
pub struct ScopeDisposer<'a> {
    f: Box<dyn FnOnce() + 'a>,
}

impl<'a> ScopeDisposer<'a> {
    fn new(f: impl FnOnce() + 'a) -> Self {
        Self { f: Box::new(f) }
    }

    /// Cleanup the resources owned by the [`Scope`].
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
    pub unsafe fn dispose(self) {
        (self.f)();
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
/// create_scope(|cx| {
///     outer = Some(cx);
/// });
/// # unsafe { disposer.dispose(); }
/// ```
///
/// # Examples
///
/// ```
/// # use sycamore_reactive::*;
/// let disposer = create_scope(|cx| {
///     // Use cx here.
/// });
/// unsafe { disposer.dispose(); }
/// ```
#[must_use = "not calling the disposer function will result in a memory leak"]
pub fn create_scope<'disposer>(f: impl for<'a> FnOnce(Scope<'a>)) -> ScopeDisposer<'disposer> {
    let cx = ScopeRaw::new();
    let boxed = Box::new(cx);
    let ptr = Box::into_raw(boxed);
    // SAFETY: Safe because heap allocated value has stable address.
    // The reference passed to f cannot possible escape the closure. We know however, that ptr
    // necessary outlives the closure call because it is only dropped in the returned disposer
    // closure.
    untrack(|| f(unsafe { BoundedScope::new(&*ptr) }));
    //                      ^^^ -> `ptr` is still accessible here after the call to f.

    // Ownership of `ptr` is passed into the closure.
    ScopeDisposer::new(move || unsafe {
        // SAFETY: Safe because ptr created using Box::into_raw.
        let boxed = Box::from_raw(ptr);
        // SAFETY: Outside of call to f.
        boxed.dispose();
    })
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
/// # create_scope_immediate(|cx| {
/// let mut outer = None;
/// let disposer = create_child_scope(cx, |cx| {
///     outer = Some(cx);
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
/// # create_scope_immediate(|cx| {
/// let mut outer = String::new();
/// let disposer = create_child_scope(cx, |cx| {
///     // outer is accessible inside the closure.
///     outer = "Hello World!".to_string();
/// });
/// unsafe { disposer.dispose(); }
/// drop(outer);
/// //   ^^^^^ -> and remains accessible outside the closure.
/// # });
/// ```
pub fn create_child_scope<'a, F>(cx: Scope<'a>, f: F) -> ScopeDisposer<'a>
where
    F: for<'child_lifetime> FnOnce(BoundedScope<'child_lifetime, 'a>),
{
    let mut child = ScopeRaw::new();
    // SAFETY: The only fields that are accessed on self from child is `context` which does not
    // have any lifetime annotations.
    child.parent = Some(unsafe { std::mem::transmute(cx.raw as *const _) });
    let boxed = Box::new(child);
    let ptr = Box::into_raw(boxed);

    let key = cx
        .raw
        .inner
        .borrow_mut()
        .child_scopes
        // SAFETY: None of the fields of ptr are accessed through child_scopes therefore we can
        // safely transmute the lifetime.
        .insert(unsafe { std::mem::transmute(ptr) });

    // SAFETY: the address of the cx lives as long as 'a because:
    // - It is allocated on the heap and therefore has a stable address.
    // - self.child_cx is append only. That means that the Box<cx> will not be dropped until Self is
    //   dropped.
    f(unsafe { BoundedScope::new(&*ptr) });
    //                                  ^^^ -> `ptr` is still accessible here after
    // the call to f.
    ScopeDisposer::new(move || unsafe {
        let cx = cx.raw.inner.borrow_mut().child_scopes.remove(key).unwrap();
        // SAFETY: Safe because ptr created using Box::into_raw and closure cannot live longer
        // than 'a.
        let cx = Box::from_raw(cx);
        // SAFETY: Outside of call to f.
        cx.dispose();
    })
}

/// Creates a reactive scope, runs the callback, and disposes the scope immediately.
///
/// Calling this is equivalent to writing:
/// ```
/// # use sycamore_reactive::*;
/// # unsafe {
/// (create_scope(|cx| {
///     // ...
/// })).dispose(); // Call the disposer function immediately
/// # }
/// ```
pub fn create_scope_immediate(f: impl for<'a> FnOnce(Scope<'a>)) {
    let disposer = create_scope(f);
    // SAFETY: We are not accessing the scope after calling the disposer function.
    unsafe {
        disposer.dispose();
    }
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
/// # create_scope_immediate(|cx| {
/// let mut outer = None;
/// let disposer = create_child_scope(cx, |cx| {
///     let data = create_ref(cx, 0);
///     let raw: &i32 = &data;
///     outer = Some(raw);
///     //           ^^^
/// });
/// disposer();
/// let _ = outer.unwrap();
/// # });
/// ```
pub fn create_ref<T>(cx: Scope, value: T) -> &T {
    cx.raw.arena.alloc(value)
}

/// Adds a callback that is called when the scope is destroyed.
pub fn on_cleanup<'a>(cx: Scope<'a>, f: impl FnOnce() + 'a) {
    cx.raw.inner.borrow_mut().cleanups.push(Box::new(f));
}

/// Returns a [`RcSignal`] that is `true` when the scope is still valid and `false` once it is
/// disposed.
pub fn use_scope_status(cx: Scope) -> RcSignal<bool> {
    let status = create_rc_signal(true);
    on_cleanup(cx, {
        let status = status.clone();
        move || status.set(false)
    });
    status
}

impl<'a> ScopeRaw<'a> {
    /// Cleanup the resources owned by the [`Scope`]. For more details, see
    /// [`ScopeDisposer::dispose`].
    ///
    /// This is automatically called in [`Drop`]
    /// However, [`dispose`](Self::dispose) only needs to take `&self` instead of `&mut self`.
    /// Dropping a [`Scope`] will automatically call [`dispose`](Self::dispose).
    pub(crate) unsafe fn dispose(&self) {
        let mut inner = self.inner.borrow_mut();
        // Drop child contexts.
        for &i in mem::take(&mut inner.child_scopes).values() {
            // SAFETY: These pointers were allocated in Self::create_child_scope.
            let cx = Box::from_raw(i);
            // Dispose of cx if it has not already been disposed.
            cx.dispose()
        }
        // Drop effects.
        drop(mem::take(&mut inner.effects));
        // Call cleanup functions in an untracked scope.
        untrack(|| {
            for cb in mem::take(&mut inner.cleanups) {
                cb();
            }
        });
        // Cleanup signals and refs allocated on the arena.
        self.arena.dispose();
    }
}

impl Drop for ScopeRaw<'_> {
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
/// # create_scope_immediate(|cx| {
/// let state = create_signal(cx, 0);
///
/// create_effect(cx, on([state], || {
///     println!("State changed. New state value = {}", state.get());
/// })); // Prints "State changed. New state value = 0"
///
/// state.set(1); // Prints "State changed. New state value = 1"
/// # });
/// ```
pub fn on<'a, U, const N: usize>(
    dependencies: [&'a (dyn AnyReadSignal<'a> + 'a); N],
    mut f: impl FnMut() -> U + 'a,
) -> impl FnMut() -> U + 'a {
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
    use super::*;

    #[test]
    fn refs() {
        let disposer = create_scope(|cx| {
            let r = create_ref(cx, 0);
            on_cleanup(cx, move || {
                let _ = r; // r can be accessed inside scope here.
                dbg!(r);
            })
        });
        unsafe {
            disposer.dispose();
        }
    }

    #[test]
    fn cleanup() {
        create_scope_immediate(|cx| {
            let cleanup_called = create_signal(cx, false);
            let disposer = create_child_scope(cx, |cx| {
                on_cleanup(cx, || {
                    cleanup_called.set(true);
                });
            });
            assert!(!*cleanup_called.get());
            unsafe {
                disposer.dispose();
            }
            assert!(*cleanup_called.get());
        });
    }

    #[test]
    fn cleanup_in_effect() {
        create_scope_immediate(|cx| {
            let trigger = create_signal(cx, ());

            let counter = create_signal(cx, 0);

            create_effect_scoped(cx, |cx| {
                trigger.track();

                on_cleanup(cx, || {
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
        create_scope_immediate(|cx| {
            let trigger = create_signal(cx, ());

            let counter = create_signal(cx, 0);

            create_effect_scoped(cx, |cx| {
                counter.set(*counter.get_untracked() + 1);

                on_cleanup(cx, || {
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
        create_scope_immediate(|cx| {
            let signal = create_signal(cx, None);
            let disposer = create_child_scope(cx, |_cx| {});
            signal.set(Some(disposer));
        });
    }
}

//! Reactive scopes.

use std::cell::RefCell;
use std::mem;
use std::rc::{Rc, Weak};

use slotmap::{new_key_type, SlotMap};

use crate::effect::{untrack, EffectState};
use crate::signal::SignalDataAny;

new_key_type! {
    pub(crate) struct ScopeKey;
}

thread_local! {
    /// The current [`ReactiveScope`] of the current thread and the key to [`SCOPES`].
    pub(crate) static CURRENT_SCOPE: RefCell<Option<ReactiveScope>> = RefCell::new(None);
    /// A slotmap of all [`ReactiveScope`]s that are currently valid in the current thread.
    ///
    /// All scopes inside the slotmap should be valid because they are tied to the lifetime of a
    /// [`ReactiveScopeInner`]. When the [`ReactiveScopeInner`] is dropped, the matching weak
    /// reference in the slotmap is removed.
    ///
    /// This is essentially a list of valid [`ReactiveScopeInner`]s using RAII.
    pub(crate) static SCOPES: RefCell<SlotMap<ScopeKey, WeakReactiveScope>> = RefCell::new(SlotMap::with_key());
}

/// Insert a scope into [`SCOPES`]. Returns the created [`ScopeKey`].
fn insert_scope(scope: WeakReactiveScope) -> ScopeKey {
    SCOPES.with(|scopes| scopes.borrow_mut().insert(scope))
}

/// Removes a scope from [`SCOPES`].
///
/// # Panics
/// This method will `panic!()` if the key is not found in [`SCOPES`].
fn remove_scope(key: ScopeKey) {
    SCOPES.with(|scopes| {
        scopes
            .borrow_mut()
            .remove(key)
            .expect("could not find scope with key")
    });
}

struct CleanupCallback(Box<dyn FnOnce()>);

#[derive(Default)]
pub(crate) struct ReactiveScopeInner {
    /// The key to the [`WeakReactiveScope`] in [`SCOPES`]. The value should always be `Some` after
    /// initialization.
    pub(crate) key: Option<ScopeKey>,
    /// The [`ReactiveScope`] owns all signals that are created within the scope.
    pub(crate) signals: Vec<Box<dyn SignalDataAny>>,
    /// The [`ReactiveScope`] owns all the effects that are created within the scope.
    effects: Vec<Rc<RefCell<Option<EffectState>>>>,
    /// Callbacks to run when the scope is dropped.
    cleanups: Vec<CleanupCallback>,
}

impl ReactiveScopeInner {
    /// Creates a new [`ReactiveScopeInner`]. Note that `key` is set to `None` by default. It is up
    /// to the responsibility of the caller to initialize `key` with the `ScopeKey` for [`SCOPES`].
    fn new() -> Self {
        Self::default()
    }
}

impl Drop for ReactiveScopeInner {
    /// Remove itself from [`SCOPES`].
    fn drop(&mut self) {
        // Run cleanup callbacks.
        for cb in mem::take(&mut self.cleanups) {
            untrack(cb.0)
        }

        // Remove self from `SCOPES`.
        let key = self.key.unwrap();
        remove_scope(key);
    }
}

/// Owns the signals, effects, and cleanup callbacks created within it.
///
/// A `ReactiveScope` can be cloned cheaply because it is backed by a `Rc` (reference-counted)
/// pointer.
#[derive(Clone)]
pub struct ReactiveScope {
    pub(crate) inner: Rc<RefCell<ReactiveScopeInner>>,
}

impl ReactiveScope {
    /// Create a new [`ReactiveScope`] and inserts it into [`SCOPES`].
    fn new() -> Self {
        let inner = Rc::new(RefCell::new(ReactiveScopeInner::new()));
        let weak = Rc::downgrade(&inner);
        let key = insert_scope(weak);

        // initialize ReactiveScopeInner.key
        inner.borrow_mut().key = Some(key);

        Self { inner }
    }

    /// Get the [`ScopeKey`] for the scope.
    pub(crate) fn key(&self) -> ScopeKey {
        self.inner.borrow().key.unwrap()
    }

    pub(crate) fn add_effect_state(&self, effect: Rc<RefCell<Option<EffectState>>>) {
        self.inner.borrow_mut().effects.push(effect);
    }

    /// Adds a callback that will be called when the [`ReactiveScope`] is dropped.
    ///
    /// If you want to add a cleanup callback to the *current* scope, use [`on_cleanup`] instead.
    pub fn add_cleanup_callback(&self, callback: impl FnOnce() + 'static) {
        self.inner
            .borrow_mut()
            .cleanups
            .push(CleanupCallback(Box::new(callback)));
    }

    /// Extends the reactive scope.
    ///
    /// Most likely you want to use this method to run some code in an outer scope rather than an
    /// inner scope.
    pub fn extend(&self, f: impl FnOnce()) {
        CURRENT_SCOPE.with(|current_scope| {
            let scope = self.clone();
            let outer = mem::replace(&mut *current_scope.borrow_mut(), Some(scope));
            f();
            mem::replace(&mut *current_scope.borrow_mut(), outer).unwrap();
        });
    }
}

pub(crate) type WeakReactiveScope = Weak<RefCell<ReactiveScopeInner>>;

/// Create a new detached [`ReactiveScope`].
#[must_use = "immediately dropping a ReactiveScope will drop all child scopes"]
pub fn create_root(f: impl FnOnce()) -> ReactiveScope {
    CURRENT_SCOPE.with(|current_scope| {
        let scope = ReactiveScope::new();
        let outer = mem::replace(&mut *current_scope.borrow_mut(), Some(scope));
        f();
        mem::replace(&mut *current_scope.borrow_mut(), outer).unwrap()
    })
}

/// Adds a cleanup callback to the current scope.
///
/// # Panics
/// This function will `panic!()` if not inside a reactive scope.
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    CURRENT_SCOPE.with(|current_scope| {
        current_scope
            .borrow()
            .as_ref()
            .expect("not inside a reactive scope")
            .add_cleanup_callback(f);
    });
}

/// Returns a shallow clone of the current scope or `None` if not inside a reactive scope.
pub fn current_scope() -> Option<ReactiveScope> {
    CURRENT_SCOPE.with(|current_scope| current_scope.borrow().clone())
}

#[cfg(test)]
mod tests {
    use crate::signal::create_signal;

    use super::*;

    #[test]
    fn cleanup() {
        let _ = create_root(|| {
            let (cleanup_called, set_cleanup_called) = create_signal(false);
            let scope = create_root(move || {
                on_cleanup(move || {
                    set_cleanup_called.set(true);
                });
            });
            assert!(!*cleanup_called.get());
            drop(scope);
            assert!(*cleanup_called.get());
        });
    }
}

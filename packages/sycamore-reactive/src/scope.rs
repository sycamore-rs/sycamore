//! Reactive scopes.

use std::cell::RefCell;
use std::mem;
#[cfg(debug_assertions)]
use std::panic::Location;
use std::rc::Rc;

use slotmap::{new_key_type, SlotMap};

use crate::effect::{untrack, EffectState};
use crate::signal::SignalDataAny;
use crate::ContextAny;

new_key_type! {
    pub(crate) struct ScopeKey;
}

thread_local! {
    /// A stack of [`ReactiveScope`] on the current thread.
    pub(crate) static SCOPE_STACK: RefCell<Vec<ReactiveScope>> = RefCell::new(Vec::new());
    /// A slotmap of all [`ReactiveScope`]s that are currently valid in the current thread.
    ///
    /// All scopes inside the slotmap should be valid because they are tied to the lifetime of a
    /// [`ReactiveScopeInner`]. When the [`ReactiveScopeInner`] is dropped, the matching weak
    /// reference in the slotmap is removed.
    ///
    /// This is essentially a list of valid [`ReactiveScopeInner`]s using RAII.
    pub(crate) static VALID_SCOPES: RefCell<SlotMap<ScopeKey, ReactiveScopeGlobalRef>> = RefCell::new(SlotMap::with_key());
}

/// Insert a scope into [`VALID_SCOPES`]. Returns the created [`ScopeKey`].
fn insert_scope(scope: ReactiveScopeGlobalRef) -> ScopeKey {
    VALID_SCOPES.with(|scopes| scopes.borrow_mut().insert(scope))
}

/// Removes a scope from [`VALID_SCOPES`].
///
/// # Panics
/// This method will `panic!()` if the key is not found in [`VALID_SCOPES`].
fn remove_scope(key: ScopeKey) {
    VALID_SCOPES.with(|scopes| {
        scopes
            .borrow_mut()
            .remove(key)
            .expect("could not find scope with key")
    });
}

struct CleanupCallback(Box<dyn FnOnce()>);

pub(crate) struct ReactiveScopeInner {
    /// The key to the [`WeakReactiveScope`] in [`VALID_SCOPES`]. The value should always be `Some` after
    /// initialization.
    pub(crate) key: Option<ScopeKey>,
    /// The [`ReactiveScope`] owns all signals that are created within the scope.
    pub(crate) signals: Vec<Box<dyn SignalDataAny>>,
    /// The [`ReactiveScope`] owns all the effects that are created within the scope.
    effects: Vec<Rc<RefCell<Option<EffectState>>>>,
    /// Callbacks to run when the scope is dropped.
    cleanups: Vec<CleanupCallback>,
    /// An optional context for the scope.
    pub(crate) context: Option<Box<dyn ContextAny>>,
    /// The source location where the scope was created. Only available in debug mode.
    ///
    /// Used when accessing the signal when scope is no longer valid to provide a useful error
    /// message.
    #[cfg(debug_assertions)]
    pub(crate) creation_loc: Location<'static>,
}

impl ReactiveScopeInner {
    /// Creates a new [`ReactiveScopeInner`]. Note that `key` is set to `None` by default. It is up
    /// to the responsibility of the caller to initialize `key` with the `ScopeKey` for [`VALID_SCOPES`].
    #[cfg_attr(debug_assertions, track_caller)]
    fn new() -> Self {
        Self {
            key: Default::default(),
            signals: Default::default(),
            effects: Default::default(),
            cleanups: Default::default(),
            context: Default::default(),
            #[cfg(debug_assertions)]
            creation_loc: *Location::caller(),
        }
    }

    /// Returns a closure that runs cleanup callbacks and destroys owned effects to trigger nested
    /// cleanup callbacks.
    fn cleanup(&mut self) -> impl FnOnce() {
        let cleanups = mem::take(&mut self.cleanups);
        let effects = mem::take(&mut self.effects);

        move || {
            // Run cleanup callbacks.
            for cb in cleanups {
                untrack(cb.0)
            }

            // Drop effects now so that the cleanup functions can still access signals from outer
            // scope.
            debug_assert!(effects.iter().all(|e| Rc::strong_count(e) == 1));
            drop(effects);
        }
    }
}

pub(crate) struct ReactiveScopeGlobalRef(pub Rc<RefCell<ReactiveScopeInner>>);

/// Owns the signals, effects, and cleanup callbacks created within it.
///
/// A `ReactiveScope` can be cloned cheaply because it is backed by a `Rc` (reference-counted)
/// pointer.
#[derive(Clone)]
pub struct ReactiveScope {
    pub(crate) inner: Rc<RefCell<ReactiveScopeInner>>,
}

impl ReactiveScope {
    /// Create a new [`ReactiveScope`] and inserts it into [`VALID_SCOPES`].
    #[cfg_attr(debug_assertions, track_caller)]
    fn new() -> Self {
        let inner = Rc::new(RefCell::new(ReactiveScopeInner::new()));
        let key = insert_scope(ReactiveScopeGlobalRef(inner.clone()));

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
        SCOPE_STACK.with(|scope_stack| {
            let scope = self.clone();
            scope_stack.borrow_mut().push(scope);
            f();
            scope_stack.borrow_mut().pop().unwrap();
        });
    }
}

impl Drop for ReactiveScope {
    fn drop(&mut self) {
        if Rc::strong_count(&self.inner) == 2 {
            // Only 2 strong refs left. One is self and the other is stored in `SCOPES`.
            // Run cleanup in ReactiveScopeInner but wait until cleanup is finished before
            // destroying ref in `SCOPES` so that cleanup functions can still access signals.
            let call_cleanup = self.inner.borrow_mut().cleanup();
            // Call the closure separately to prevent a RefCell borrow error.
            //
            // The cleanup function can potentially call `.borrow()` on the scope which will panic
            // since it is `borrow_mut()`ed previously.
            call_cleanup();

            // Remove self from `SCOPES`.
            remove_scope(self.key());
        }
    }
}

/// Create a new detached [`ReactiveScope`].
#[must_use = "immediately dropping a ReactiveScope will drop all child scopes"]
#[track_caller]
pub fn create_root(f: impl FnOnce()) -> ReactiveScope {
    let scope = ReactiveScope::new();
    SCOPE_STACK.with(|scope_stack| {
        scope_stack.borrow_mut().push(scope);
        f();
        scope_stack.borrow_mut().pop().unwrap()
    })
}

/// Adds a cleanup callback to the current scope.
///
/// # Panics
/// This function will `panic!()` if not inside a reactive scope.
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    SCOPE_STACK.with(|current_scope| {
        current_scope
            .borrow()
            .last()
            .expect("not inside a reactive scope")
            .add_cleanup_callback(f);
    });
}

/// Returns a shallow clone of the current scope or `None` if not inside a reactive scope.
pub fn current_scope() -> Option<ReactiveScope> {
    SCOPE_STACK.with(|scope_stack| scope_stack.borrow().last().cloned())
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

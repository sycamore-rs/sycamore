//! Reactive signals.

use std::any::Any;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
#[cfg(debug_assertions)]
use std::panic::Location;
use std::rc::{Rc, Weak};

use indexmap::IndexMap;

use crate::effect::{EffectState, EffectStatePtr, CURRENT_LISTENER};
use crate::scope::{ScopeKey, SCOPE_STACK, VALID_SCOPES};

/// An error returned by [`ReadSignal::get`] and [`WriteSignal::set`].
#[derive(Debug, Clone)]
pub struct ScopeDestroyedError {
    #[cfg(debug_assertions)]
    creation_loc: Location<'static>,
    #[cfg(debug_assertions)]
    scope_creation_loc: Location<'static>,
}

impl Error for ScopeDestroyedError {}

impl Display for ScopeDestroyedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "reactive scope for signal already destroyed")?;
        #[cfg(debug_assertions)]
        {
            writeln!(f, "signal created at {}", self.creation_loc)?;
            writeln!(f, "inside scope created at {}", self.scope_creation_loc)?;
        }

        Ok(())
    }
}

/// Backing storage for a signal.
pub(crate) struct SignalData<T> {
    inner: Rc<T>,
    dependents: IndexMap<EffectStatePtr, Weak<RefCell<Option<EffectState>>>>,
}

/// Explicitly implement `Clone` to prevent type bounds on `T`.
impl<T> Clone for SignalData<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            dependents: self.dependents.clone(),
        }
    }
}

/// A trait for any `SignalData<T>`.
pub(crate) trait SignalDataAny: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn subscribe(&mut self, effect: Weak<RefCell<Option<EffectState>>>);
    fn unsubscribe(&mut self, ptr: EffectStatePtr);
    #[must_use]
    fn clone_dependents(&self) -> IndexMap<EffectStatePtr, Weak<RefCell<Option<EffectState>>>>;
}

impl<T: 'static> SignalDataAny for SignalData<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn subscribe(&mut self, effect: Weak<RefCell<Option<EffectState>>>) {
        self.dependents.insert(Weak::as_ptr(&effect), effect);
    }
    fn unsubscribe(&mut self, ptr: EffectStatePtr) {
        self.dependents.remove(&ptr);
    }
    fn clone_dependents(&self) -> IndexMap<EffectStatePtr, Weak<RefCell<Option<EffectState>>>> {
        self.dependents.clone()
    }
}

impl<T> Drop for SignalData<T> {
    fn drop(&mut self) {
        // TODO: Remove self from all effect dependencies.
    }
}

/// Data needed to refer to a SignalData.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SignalId {
    /// Key to the reactive scope in the slab. Note that accessing the value pointed to by this key
    /// is not enough to ensure that the `ReadSignal` is still valid. One must also check that the
    /// `scope_id` also matches.
    scope_key: ScopeKey,
    /// Index of the signal in the reactive scope's signal array.
    signal_index: usize,
    /// The source location where the signal was created. Only available in debug mode.
    ///
    /// Used when accessing the signal when scope is no longer valid to provide a useful error
    /// message.
    #[cfg(debug_assertions)]
    creation_loc: Location<'static>,
    /// The source location where the enclosing scope was created. Only available in debug mode.
    ///
    /// Used when accessing the signal when scope is no longer valid to provide a useful error
    /// message.
    #[cfg(debug_assertions)]
    scope_creation_loc: Location<'static>,
}

impl SignalId {
    pub fn get<Out>(self, f: impl FnOnce(Option<&dyn SignalDataAny>) -> Out) -> Out {
        VALID_SCOPES.with(|scopes| {
            let scopes = scopes.borrow();
            let scope = scopes.get(self.scope_key);
            if scope.is_none() {
                return f(None);
            }
            let scope = scope.unwrap().0.borrow();
            let data = scope.signals[self.signal_index].as_ref();
            f(Some(data))
        })
    }

    pub fn get_mut<Out>(self, f: impl FnOnce(Option<&mut dyn SignalDataAny>) -> Out) -> Out {
        VALID_SCOPES.with(|scopes| {
            let scopes = scopes.borrow();
            let scope = scopes.get(self.scope_key);
            if scope.is_none() {
                return f(None);
            }
            let mut scope = scope.unwrap().0.borrow_mut();
            let data = scope.signals[self.signal_index].as_mut();
            f(Some(data))
        })
    }
}

/// A `ReadSignal` is a handle to some reactive state allocated in the current reactive scope.
pub struct ReadSignal<T> {
    id: SignalId,
    /// Use `*const T` instead of `T` to prevent drop check.
    _phantom: PhantomData<*const T>,
}

/// Explicitly implement `Clone` + `Copy` to prevent type bounds on `T`.
impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ReadSignal<T> {}

impl<T: PartialEq + 'static> PartialEq for ReadSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_untracked().eq(&other.get_untracked())
    }
}
impl<T: Eq + 'static> Eq for ReadSignal<T> {}

impl<T: Hash + 'static> Hash for ReadSignal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_untracked().hash(state);
    }
}

impl<T: 'static> ReadSignal<T> {
    /// Gets the value of the signal. If called inside a listener, will add itself as a dependency.
    ///
    /// # Panics
    /// This method will `panic!()` if the [`ReactiveScope`](crate::effect::ReactiveScope) that owns
    /// the [`ReadSignal`] is no longer valid.
    #[track_caller]
    pub fn get(self) -> Rc<T> {
        // If inside an effect, add this signal to dependency list.
        // If running inside a destructor, do nothing.
        let _ = CURRENT_LISTENER.try_with(|current_listener| {
            if let Some(listener) = current_listener.borrow().as_ref() {
                listener.add_dependency(self.id);
            }
        });

        self.get_untracked()
    }

    /// Gets the value of the signal. Does not perform any tracking.
    ///
    /// # Panics
    /// This method will `panic!()` if the [`ReactiveScope`](crate::effect::ReactiveScope) that owns
    /// the [`ReadSignal`] is no longer valid.
    #[track_caller]
    pub fn get_untracked(self) -> Rc<T> {
        let data = self.id.get(|data| {
            data.map(|data| {
                Rc::clone(
                    &data
                        .as_any()
                        .downcast_ref::<SignalData<T>>()
                        .expect("SignalData should have correct type")
                        .inner,
                )
            })
        });
        match data {
            Some(data) => data,
            None => panic!(
                "{}",
                ScopeDestroyedError {
                    #[cfg(debug_assertions)]
                    creation_loc: self.id.creation_loc,
                    #[cfg(debug_assertions)]
                    scope_creation_loc: self.id.scope_creation_loc,
                }
            ),
        }
    }
}

/// A `WriteSignal` is a handle to set some reactive data.
pub struct WriteSignal<T> {
    id: SignalId,
    /// Use `*const T` instead of `T` to prevent drop check.
    _phantom: PhantomData<*const T>,
}

/// Explicitly implement `Clone` + `Copy` to prevent type bounds on `T`.
impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for WriteSignal<T> {}

impl<T: 'static> WriteSignal<T> {
    /// Updates the value of the signal and triggers all dependents.
    ///
    /// # Panics
    /// This method will `panic!()` if the [`ReactiveScope`](crate::effect::ReactiveScope) that owns
    /// the [`ReadSignal`] is no longer valid.
    #[track_caller]
    pub fn set(self, value: T) {
        let mut dependents = None;
        let success = self
            .id
            .get_mut(|data| {
                data.map(|data| {
                    data.as_any_mut()
                        .downcast_mut::<SignalData<T>>()
                        .expect("SignalData should have correct type")
                        .inner = Rc::new(value);
                    dependents = Some(data.clone_dependents());
                })
            })
            .is_some();
        if !success {
            panic!(
                "{}",
                ScopeDestroyedError {
                    #[cfg(debug_assertions)]
                    creation_loc: self.id.creation_loc,
                    #[cfg(debug_assertions)]
                    scope_creation_loc: self.id.scope_creation_loc,
                }
            )
        }

        // Rerun all effects that depend on this signal.
        // Reverse order to re-run outer effects before inner effects.
        for dependent in dependents.unwrap().values().rev() {
            // Effect might have already been destroyed.
            if let Some(effect) = dependent.upgrade() {
                // Clone the callback to prevent holding a borrow to the EffectState.
                let callback = Rc::clone(&effect.borrow().as_ref().unwrap().callback);
                callback.borrow_mut()();
            }
        }
    }
}

/// Creates a new signal with the given value.
///
/// # Panics
/// This function will `panic!()` if it is used outside of a reactive scope.
///
/// # Example
/// ```
/// # use sycamore_reactive2::scope::create_root;
/// # use sycamore_reactive2::signal::create_signal;
/// # let _ = create_root(|| {
/// let (state, set_state) = create_signal(0);
/// assert_eq!(*state.get(), 0);
/// set_state.set(1);
/// assert_eq!(*state.get(), 1);
/// # });
/// ```
#[track_caller]
pub fn create_signal<T: 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let scope = SCOPE_STACK
        .with(|scope_stack| scope_stack.borrow().last().cloned())
        .expect("create_signal must be used inside a ReactiveScope");

    let data = SignalData {
        inner: Rc::new(value),
        dependents: IndexMap::new(),
    };
    let scope_key = scope.key();
    let signal_index = scope.inner.borrow().signals.len();
    scope.inner.borrow_mut().signals.push(Box::new(data));

    let signal_id = SignalId {
        scope_key,
        signal_index,
        #[cfg(debug_assertions)]
        creation_loc: *Location::caller(),
        #[cfg(debug_assertions)]
        scope_creation_loc: scope.inner.borrow().creation_loc,
    };

    (
        ReadSignal {
            id: signal_id,
            _phantom: PhantomData,
        },
        WriteSignal {
            id: signal_id,
            _phantom: PhantomData,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::scope::create_root;

    use super::*;

    #[test]
    fn signal_read_write() {
        let _ = create_root(|| {
            let (state, set_state) = create_signal(0);
            assert_eq!(*state.get(), 0);
            set_state.set(1);
            assert_eq!(*state.get(), 1);
        });
    }

    #[test]
    fn signal_read_outside_alive_scope() {
        let mut get_state = None;
        let root = create_root(|| {
            let (state, _) = create_signal(0);
            get_state = Some(state);
        });

        get_state.unwrap().get(); // root is still active

        drop(root);
    }

    #[test]
    #[should_panic(expected = "reactive scope for signal already destroyed")]
    fn signal_read_with_scope_already_destroyed() {
        let mut get_state = None;
        let _ = create_root(|| {
            let (state, _) = create_signal(0);
            get_state = Some(state);
        });

        get_state.unwrap().get();
    }
}

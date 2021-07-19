//! Reactive signals.

use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use indexmap::IndexMap;

use crate::effect::{EffectState, EffectStatePtr};
use crate::scope::{ScopeKey, CURRENT_SCOPE, SCOPES};

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
    fn trigger_update(&self);
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
    fn trigger_update(&self) {
        for dependent in self.dependents.values() {
            dependent.upgrade().unwrap().borrow().as_ref().unwrap().trigger();
        }
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
}

impl SignalId {
    pub fn get<Out>(self, f: impl FnOnce(Option<&dyn SignalDataAny>) -> Out) -> Out {
        SCOPES.with(|scopes| {
            let scopes = scopes.borrow();
            let scope = scopes.get(self.scope_key);
            if scope.is_none() {
                return f(None);
            }
            let scope = scope
                .unwrap()
                .upgrade()
                .expect("weak reference should always be valid");
            let scope = scope.borrow();
            let data = scope.signals[self.signal_index].as_ref();
            f(Some(data))
        })
    }

    pub fn get_mut<Out>(self, f: impl FnOnce(Option<&mut dyn SignalDataAny>) -> Out) -> Out {
        SCOPES.with(|scopes| {
            let scopes = scopes.borrow();
            let scope = scopes.get(self.scope_key);
            if scope.is_none() {
                return f(None);
            }
            let scope = scope
                .unwrap()
                .upgrade()
                .expect("weak reference should always be valid");
            let mut scope = scope.borrow_mut();
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

impl<T: 'static> ReadSignal<T> {
    /// Gets the value of the signal.
    ///
    /// # Panics
    /// This method will `panic!()` if the [`ReactiveScope`](crate::effect::ReactiveScope) that owns
    /// the [`ReadSignal`] is no longer valid.
    #[track_caller]
    pub fn get(self) -> Rc<T> {
        self.id.get(|data| match data {
            Some(data) => Rc::clone(
                &data
                    .as_any()
                    .downcast_ref::<SignalData<T>>()
                    .expect("SignalData should have correct type")
                    .inner,
            ),
            None => panic!("reactive scope for signal already destroyed"),
        })
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
    pub fn set(self, value: T) {
        self.id.get_mut(|data| match data {
            Some(data) => {
                data.as_any_mut()
                    .downcast_mut::<SignalData<T>>()
                    .expect("SignalData should have correct type")
                    .inner = Rc::new(value);
                data.trigger_update();
            }
            None => panic!("reactive scope for signal already destroyed"),
        });
    }
}

/// Creates a new signal with the given value.
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
pub fn create_signal<T: 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    CURRENT_SCOPE.with(|current_scope| {
        let scope = current_scope
            .borrow()
            .clone()
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
    })
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

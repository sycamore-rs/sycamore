use std::marker::PhantomData;
use std::rc::Rc;

use crate::effect::{CURRENT_SCOPE, SCOPES};

/// Backing storage for a signal.
pub(crate) struct SignalData<T> {
    inner: Rc<T>,
}

/// Explicitly implement `Clone` to prevent type bounds on `T`.
impl<T> Clone for SignalData<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Data needed to refer to a SignalData.
#[derive(Clone, Copy)]
struct SignalId {
    /// Key to the reactive scope in the slab. Note that accessing the value pointed to by this key
    /// is not enough to ensure that the `ReadSignal` is still valid. One must also check that the
    /// `scope_id` also matches.
    scope_key: usize,
    /// Id of the reactive scope.
    scope_id: usize,
    /// Index of the signal in the reactive scope's signal array.
    signal_index: usize,
}

/// A `ReadSignal` is an accessor to some reactive state allocated in the current reactive scope.
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
        SCOPES.with(|scopes| {
            let scopes = scopes.borrow();
            let scope = scopes.get(self.id.scope_key);
            if scope.is_none() || scope.unwrap().id() != self.id.scope_id {
                panic!("reactive scope for signal already destroyed");
            }
            let data = &scope.unwrap().signals().borrow()[self.id.signal_index];
            Rc::clone(
                &data
                    .downcast_ref::<SignalData<T>>()
                    .expect("SignalData should have correct type")
                    .inner,
            )
        })
    }
}

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
    /// Updates the value of the signal.
    ///
    /// TODO: trigger dependencies
    ///
    /// # Panics
    /// This method will `panic!()` if the [`ReactiveScope`](crate::effect::ReactiveScope) that owns
    /// the [`ReadSignal`] is no longer valid.
    pub fn set(self, value: T) {
        SCOPES.with(|scopes| {
            let scopes = scopes.borrow_mut();
            let scope = scopes.get(self.id.scope_key);
            if scope.is_none() || scope.unwrap().id() != self.id.scope_id {
                panic!("reactive scope for signal already destroyed");
            }
            let data = &mut scope.unwrap().signals().borrow_mut()[self.id.signal_index];
            let data = data
                .downcast_mut::<SignalData<T>>()
                .expect("SignalData should have correct type");

            data.inner = Rc::new(value);
        });
    }
}

pub fn create_signal<T: 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    CURRENT_SCOPE.with(|current_scope| {
        let scope = current_scope
            .borrow()
            .clone()
            .expect("create_signal must be used inside a ReactiveScope");

        let data = SignalData {
            inner: Rc::new(value),
        };
        let scope_id = scope.0.id();
        let scope_key = scope.1;
        let signal_index = scope.0.signals().borrow().len();
        scope.0.signals().borrow_mut().push(Box::new(data));

        let signal_id = SignalId {
            scope_key,
            scope_id,
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
    use crate::effect::create_root_scope;

    use super::*;

    #[test]
    fn signal_read_write() {
        create_root_scope(|| {
            let (state, set_state) = create_signal(0);
            assert_eq!(*state.get(), 0);
            set_state.set(1);
            assert_eq!(*state.get(), 1);
        });
    }
}

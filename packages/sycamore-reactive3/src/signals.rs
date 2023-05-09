//! Reactive signals.

use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;

use slotmap::new_key_type;

use crate::{Root, Scope};

new_key_type! { pub(crate) struct SignalId; }

/// Stores al the data associated with a signal.
pub(crate) struct SignalState {
    pub value: RefCell<Box<dyn Any>>,
    /// List of signals whose value this signal depends on.
    ///
    /// If any of the dependency signals are updated, this signal will automatically be updated as
    /// well.
    pub dependencies: Vec<SignalId>,
    /// List of signals which depend on the value of this signal.
    ///
    /// If this signal updates, any dependent signal will automatically be updated as well.
    pub dependents: Vec<SignalId>,
    /// A callback that automatically updates the value of the signal when one of its dependencies
    /// updates.
    ///
    /// A signal created using [`create_signal`] can be thought of as a signal which is never
    /// autoamtically updated. A signal created using [`create_memo`] can be thought of as a signal
    /// that is always automatically updated.
    ///
    /// Note that the update function takes a `&mut dyn Any`. The update function should only ever
    /// set this value to the same type as the signal.
    pub update: Option<Box<dyn FnMut(&mut Box<dyn Any>)>>,
}

pub struct Signal<T: 'static> {
    pub(crate) id: SignalId,
    root: &'static Root,
    _phantom: PhantomData<T>,
}

pub fn create_signal<T>(cx: Scope, value: T) -> Signal<T> {
    let data = SignalState {
        value: RefCell::new(Box::new(value)),
        dependencies: Vec::new(),
        dependents: Vec::new(),
        update: None,
    };
    let key = cx.root.signals.borrow_mut().insert(data);
    // Add the signal the scope signal list so that it is properly dropped when the scope is
    // dropped.
    cx.get_data(|cx| cx.signals.push(key));
    Signal {
        id: key,
        root: cx.root,
        _phantom: PhantomData,
    }
}

impl<T> Signal<T> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data<U>(self, f: impl FnOnce(&SignalState) -> U) -> U {
        f(&mut self
            .root
            .signals
            .borrow()
            .get(self.id)
            .expect("signal is disposed"))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data_mut<U>(self, f: impl FnOnce(&mut SignalState) -> U) -> U {
        f(&mut self
            .root
            .signals
            .borrow_mut()
            .get_mut(self.id)
            .expect("signal is disposed"))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_untracked(self) -> T
    where
        T: Copy,
    {
        self.with_untracked(|value| *value)
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_clone_untracked(self) -> T
    where
        T: Clone,
    {
        self.with_untracked(Clone::clone)
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get(self) -> T
    where
        T: Copy,
    {
        self.track();
        self.get_untracked()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_clone(self) -> T
    where
        T: Clone,
    {
        self.track();
        self.get_clone_untracked()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(self, new: T) -> T {
        self.update(|val| std::mem::replace(val, new))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn with_untracked<U>(self, f: impl FnOnce(&T) -> U) -> U {
        self.get_data(|signal| {
            f(signal
                .value
                .borrow()
                .downcast_ref::<T>()
                .expect("wrong signal type in slotmap"))
        })
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn with<U>(self, f: impl FnOnce(&T) -> U) -> U {
        self.track();
        self.with_untracked(f)
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn update<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        let ret = self.get_data(|signal| {
            f(signal
                .value
                .borrow_mut()
                .downcast_mut()
                .expect("wrong signal type in slotmap"))
        });
        self.root.propagate_updates(self.id);
        ret
    }

    pub fn track(self) {
        if let Some(tracker) = &mut *self.root.tracker.borrow_mut() {
            tracker.dependencies.push(self.id);
        }
    }
}

/// We manually implement `Clone` + `Copy` for `Signal` so that we don't get extra bounds on `T`.
impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            root: self.root,
            _phantom: self._phantom,
        }
    }
}
impl<T> Copy for Signal<T> {}

#[cfg(feature = "nightly")]
impl<T: Copy> FnOnce<()> for Signal<T> {
    type Output = T;

    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.get()
    }
}

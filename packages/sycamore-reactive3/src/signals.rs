//! Reactive signals.

use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;

use slotmap::new_key_type;

use crate::{Root, Scope};

new_key_type! { pub(crate) struct SignalKey; }

pub(crate) struct SignalData {
    value: RefCell<Box<dyn Any>>,
}

#[derive(Clone, Copy)]
pub struct Signal<T: 'static> {
    key: SignalKey,
    root: &'static Root,
    _phantom: PhantomData<T>,
}

pub fn create_signal<T>(cx: Scope, value: T) -> Signal<T> {
    let data = SignalData {
        value: RefCell::new(Box::new(value)),
    };
    let key = cx.root.signals.borrow_mut().insert(data);
    // Add the signal the scope signal list so that it is properly dropped when the scope is
    // dropped.
    cx.get_data(|cx| cx.signals.push(key));
    Signal {
        key,
        root: cx.root,
        _phantom: PhantomData,
    }
}

impl<T> Signal<T> {
    fn get_data<U>(self, f: impl FnOnce(&mut SignalData) -> U) -> U {
        f(&mut self.root.signals.borrow_mut()[self.key])
    }

    pub fn get(self) -> T
    where
        T: Copy,
    {
        self.with(|value| *value)
    }

    pub fn get_clone(self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    pub fn set(self, new: T) -> T {
        self.get_data(|signal| {
            *signal
                .value
                .replace(Box::new(new))
                .downcast()
                .expect("wrong signal type in slotmap")
        })
    }

    pub fn with<U>(self, f: impl FnOnce(&T) -> U) -> U {
        self.get_data(|signal| {
            f(signal
                .value
                .borrow()
                .downcast_ref::<T>()
                .expect("wrong signal type in slotmap"))
        })
    }

    pub fn update(self, f: impl FnOnce(&mut T)) {
        self.get_data(|signal| {
            f(signal
                .value
                .borrow_mut()
                .downcast_mut()
                .expect("wrong signal type in slotmap"))
        });
    }
}

#[cfg(feature = "nightly")]
impl<T: Copy> FnOnce<()> for Signal<T> {
    type Output = T;
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.get()
    }
}

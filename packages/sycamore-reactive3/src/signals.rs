//! Reactive signals.

use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;

use slotmap::new_key_type;

use crate::Scope;

new_key_type! { pub(crate) struct SignalKey; }

pub(crate) struct SignalData {
    value: RefCell<Box<dyn Any>>,
}

#[derive(Clone, Copy)]
pub struct Signal<T: 'static> {
    key: SignalKey,
    cx: Scope,
    _phantom: PhantomData<T>,
}

pub fn create_signal<T>(cx: Scope, value: T) -> Signal<T> {
    let data = SignalData {
        value: RefCell::new(Box::new(value)),
    };
    let key = cx.get_data(|cx| cx.signals.insert(data));
    Signal {
        key,
        cx,
        _phantom: PhantomData,
    }
}

impl<T> Signal<T> {
    fn get_data<U>(self, f: impl FnOnce(&mut SignalData) -> U) -> U {
        self.cx.get_data(|cx| f(&mut cx.signals[self.key]))
    }

    pub fn get(self) -> T
    where
        T: Copy,
    {
        self.get_data(|signal| {
            *signal
                .value
                .borrow()
                .downcast_ref::<T>()
                .expect("wrong signal type in slotmap")
        })
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
}

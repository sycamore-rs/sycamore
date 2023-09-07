//! Stores: easy nested recursive data.

use std::cell::RefCell;

use crate::Scope;

pub struct Store<T: State> {
    value: RefCell<T>,
    trigger: T::Trigger,
}

impl<T: State> Store<T> {
    /// Internal method for implementing the `get!` macro.
    #[doc(hidden)]
    pub fn __with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&self.value.borrow())
    }

    #[doc(hidden)]
    pub fn __with_mut<U>(&self, f: impl FnOnce(&mut T) -> U) -> U {
        f(&mut self.value.borrow_mut())
    }

    /// Internal method for implementing the `get!` macro.
    #[doc(hidden)]
    pub fn __trigger(&self) -> &T::Trigger {
        &self.trigger
    }
}

pub fn create_store<T: State>(cx: Scope, value: T) -> Store<T> {
    Store {
        value: RefCell::new(value),
        trigger: T::Trigger::new(cx),
    }
}

pub struct StoreLens<T> {
    access: Box<dyn Fn() -> T>,
}

pub trait State {
    /// The type of the struct containing all the triggers for fine-grained reactivity.
    type Trigger: StateTrigger;
}

pub trait StateTrigger {
    fn new(cx: Scope) -> Self;
}

#[cfg(test)]
mod tests {
    use sycamore_reactive_macro::{get, set, State};

    use super::*;
    use crate::create_root;

    #[test]
    fn test_derive() {
        #[derive(State)]
        struct Foo {
            value: i32,
        }

        let _ = create_root(|cx| {
            let foo = create_store(cx, Foo { value: 123 });
            set!(foo.value, 456);
            let test = get!(foo.value);
            panic!("test = {test}");
        });
    }
}

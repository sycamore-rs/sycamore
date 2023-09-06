//! Stores: easy nested recursive data.

use crate::Scope;

pub struct Store<T: State> {
    value: T,
    trigger: T::Trigger,
}

impl<T: State> Store<T> {
    /// Internal method for implementing the `get!` macro.
    #[doc(hidden)]
    pub fn __get(&self) -> &T {
        &self.value
    }
}

pub fn create_store<T: State>(cx: Scope, value: T) -> Store<T> {
    Store {
        value,
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
    use sycamore_reactive_macro::{get, State};

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
            let test = get!(foo.value);
            panic!("test = {test}");
        });
    }
}

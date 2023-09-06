//! Stores: easy nested recursive data.

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

pub struct StoreLens<T> {
    access: Box<dyn Fn() -> T>,
}

pub trait State {
    /// The type of the struct containing all the triggers for fine-grained reactivity.
    type Trigger;
}

#[cfg(test)]
mod tests {
    use sycamore_reactive_macro::{get, State};

    use super::*;

    #[test]
    fn test_derive() {
        #[derive(State)]
        struct Foo {
            value: i32,
        }

        let foo = Foo { value: 123 };
        let test = get!(foo.value);
        panic!("test = {test}");
    }
}

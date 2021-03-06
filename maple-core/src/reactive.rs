//! Reactive signals.

use std::cell::RefCell;
use std::rc::Rc;

pub struct Signal<T> {
    value: Rc<T>,
    observers: Vec<Computation<T>>,
}

impl<T> Signal<T> {
    fn new(value: T) -> Self {
        Self {
            value: Rc::new(value),
            observers: Vec::new(),
        }
    }

    fn observe(&mut self, handler: Box<dyn FnMut(Rc<T>)>) {
        let mut computation = Computation(handler);
        computation.0(self.value.clone()); // call computation when added

        self.observers.push(computation);
    }

    fn update(&mut self, new_value: T) {
        self.value = Rc::new(new_value);

        // call all observers
        for observer in &mut self.observers {
            observer.0(self.value.clone());
        }
    }
}

/// A derived computation from a signal. Takes the new value as an input.
pub struct Computation<T>(Box<dyn FnMut(Rc<T>)>);

pub fn create_signal<T: 'static>(value: T) -> (impl Fn() -> Rc<T>, impl FnMut(T)) {
    let signal = Rc::new(RefCell::new(Signal::new(value)));

    let getter = {
        let signal = signal.clone();
        move || {
            // TODO: if inside an effect, add this signal to dependency list

            signal.borrow().value.clone()
        }
    };

    let setter = {
        let signal = signal.clone();
        move |new_value| {
            signal.borrow_mut().update(new_value);
        }
    };

    (getter, setter)
}

#[cfg(test)]
mod tests {
    use super::*;
}

//! Reactive signals.

use std::cell::RefCell;
use std::rc::Rc;

pub struct Signal<T> {
    inner: Rc<T>,
    observers: Vec<Rc<RefCell<Computation>>>,
}

impl<T> Signal<T> {
    fn new(value: T) -> Self {
        Self {
            inner: Rc::new(value),
            observers: Vec::new(),
        }
    }

    fn observe(&mut self, handler: Rc<RefCell<Computation>>) {
        self.observers.push(handler);
    }

    fn update(&mut self, new_value: T) {
        self.inner = Rc::new(new_value);
    }

    fn trigger_observers(&self) {
        for observer in &self.observers {
            observer.borrow_mut().0();
        }
    }
}

/// A derived computation from a signal. Takes the new value as an input.
pub struct Computation(Box<dyn FnMut()>);

thread_local! {
    static HANDLER: RefCell<Option<Rc<RefCell<Computation>>>> = RefCell::new(None);

    /// To add the dependencies, iterate through functions and execute them.
    static DEPENDENCIES: RefCell<Option<Vec<Box<dyn Fn()>>>> = RefCell::new(None);
}

pub fn create_signal<T: 'static>(value: T) -> (impl Fn() -> Rc<T>, impl Fn(T)) {
    let signal = Rc::new(RefCell::new(Signal::new(value)));

    let getter = {
        let signal = signal.clone();
        move || {
            // if inside an effect, add this signal to dependency list
            DEPENDENCIES.with(|dependencies| {
                if dependencies.borrow().is_some() {
                    let signal = signal.clone();
                    let handler =
                        HANDLER.with(|handler| handler.borrow().as_ref().unwrap().clone());

                    dependencies
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        .push(Box::new(move || {
                            signal.borrow_mut().observe(handler.clone())
                        }));
                }
            });

            signal.borrow().inner.clone()
        }
    };

    let setter = {
        let signal = signal.clone();
        move |new_value| {
            signal.borrow_mut().update(new_value);
            signal.borrow().trigger_observers();
        }
    };

    (getter, setter)
}

pub fn create_effect<F>(effect: F)
where
    F: FnMut() + 'static,
{
    DEPENDENCIES.with(|dependencies| {
        if dependencies.borrow().is_some() {
            unimplemented!("nested effects are not supported")
        }

        let effect = Rc::new(RefCell::new(Computation(Box::new(effect))));

        *dependencies.borrow_mut() = Some(Vec::new());
        HANDLER.with(|handler| *handler.borrow_mut() = Some(effect.clone()));

        // run effect for the first time to attach all the dependencies
        effect.borrow_mut().0();

        // attach dependencies
        for dependency in dependencies.borrow().as_ref().unwrap() {
            dependency();
        }

        // Reset dependencies for next effect hook
        *dependencies.borrow_mut() = None;
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn signals() {
        let (state, set_state) = create_signal(0);
        assert_eq!(*state(), 0);

        set_state(1);
        assert_eq!(*state(), 1);
    }

    #[test]
    fn effects() {
        let (state, set_state) = create_signal(0);
        assert_eq!(*state(), 0);

        let double = Rc::new(Cell::new(0));
        create_effect({
            let double = double.clone();
            move || {
                double.set(*state() * 2);
            }
        });

        set_state(1);
        assert_eq!(double.get(), 2);
        set_state(2);
        assert_eq!(double.get(), 4);
    }
}

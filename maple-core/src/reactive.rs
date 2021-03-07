//! Reactive primitives.

use std::cell::RefCell;
use std::rc::Rc;

/// Returned by functions that provide a handle to access state.
pub type StateHandle<T> = Rc<dyn Fn() -> Rc<T>>;

/// Returned by functions that provide a closure to modify state.
pub type SetStateHandle<T> = Rc<dyn Fn(T)>;

struct Signal<T> {
    inner: Rc<T>,
    observers: Vec<Rc<Computation>>,
}

impl<T> Signal<T> {
    fn new(value: T) -> Self {
        Self {
            inner: Rc::new(value),
            observers: Vec::new(),
        }
    }

    fn observe(&mut self, handler: Rc<Computation>) {
        // make sure handler is not already in self.observers
        if self
            .observers
            .iter()
            .find(|observer| {
                observer.as_ref() as *const Computation == handler.as_ref() as *const Computation
                /* do reference equality */
            })
            .is_none()
        {
            self.observers.push(handler);
        }
    }

    fn update(&mut self, new_value: T) {
        self.inner = Rc::new(new_value);
    }

    fn trigger_observers(&self) {
        for observer in &self.observers {
            observer.0();
        }
    }
}

/// A derived computation from a signal.
struct Computation(Box<dyn Fn()>);

thread_local! {
    static HANDLER: RefCell<Option<Rc<Computation>>> = RefCell::new(None);

    /// To add the dependencies, iterate through functions and execute them.
    static DEPENDENCIES: RefCell<Option<Vec<Box<dyn Fn()>>>> = RefCell::new(None);
}

/// Creates a new signal.
/// The function will return a pair of getter/setters to modify the signal and update corresponding dependencies.
///
/// # Example
/// ```rust
/// use maple_core::prelude::*;
///
/// let (state, set_state) = create_signal(0);
/// assert_eq!(*state(), 0);
///
/// set_state(1);
/// assert_eq!(*state(), 1);
/// ```
pub fn create_signal<T: 'static>(value: T) -> (StateHandle<T>, SetStateHandle<T>) {
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
            match signal.try_borrow_mut() {
                Ok(mut signal) => signal.update(new_value),
                // If the signal is already borrowed, that means it is borrowed in the getter, thus creating a cyclic dependency.
                Err(_err) => panic!("cannot create cyclic dependency"),
            };
            signal.borrow().trigger_observers();
        }
    };

    (Rc::new(getter), Rc::new(setter))
}

/// Creates an effect on signals used inside the effect closure.
pub fn create_effect<F>(effect: F)
where
    F: Fn() + 'static,
{
    DEPENDENCIES.with(|dependencies| {
        if dependencies.borrow().is_some() {
            unimplemented!("nested dependencies are not supported")
        }

        let effect = Rc::new(Computation(Box::new(effect)));

        *dependencies.borrow_mut() = Some(Vec::new());
        HANDLER.with(|handler| *handler.borrow_mut() = Some(effect.clone()));

        // run effect for the first time to attach all the dependencies
        effect.0();

        // attach dependencies
        for dependency in dependencies.borrow().as_ref().unwrap() {
            dependency();
        }

        // Reset dependencies for next effect hook
        *dependencies.borrow_mut() = None;
    })
}

/// Prevents tracking dependencies inside the closure. If called outside a reactive context, does nothing.
///
/// # Example
/// ```rust
/// use maple_core::prelude::*;
///
/// let (state, set_state) = create_signal(1);
///
/// let double = create_memo(move || untracked(|| *state()) * 2);
///
/// assert_eq!(*double(), 2);
///
/// set_state(2);
/// assert_eq!(*double(), 2); // double value should still be old value because state() was inside untracked
/// ```
pub fn untracked<F, Out>(f: F) -> Out
where
    F: Fn() -> Out,
{
    let tmp = DEPENDENCIES.with(|dependencies| dependencies.take());
    let out = f();
    DEPENDENCIES.with(|dependencies| *dependencies.borrow_mut() = tmp);

    out
}

/// Creates a memoized value from some signals. Also know as "derived stores".
pub fn create_memo<F, Out>(derived: F) -> StateHandle<Out>
where
    F: Fn() -> Out + 'static,
    Out: Clone + 'static,
{
    let derived = Rc::new(derived);
    let (memo, set_memo) = create_signal(None);

    create_effect({
        let derived = derived.clone();
        move || {
            set_memo(Some(derived()));
        }
    });

    // return memoized result
    let memo_result = move || Rc::new(Option::as_ref(&memo()).unwrap().clone());
    Rc::new(memo_result)
}

/// Creates a memoized value from some signals. Also know as "derived stores".
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is the same.
/// That is why the output of the function must implement `PartialEq`.
pub fn create_selector<F, Out>(derived: F) -> StateHandle<Out>
where
    F: Fn() -> Out + 'static,
    Out: Clone + PartialEq + std::fmt::Debug + 'static,
{
    let derived = Rc::new(derived);
    let (memo, set_memo) = create_signal(None);

    create_effect({
        let derived = derived.clone();
        let memo = memo.clone();
        move || {
            let new_value = Some(derived());
            if *untracked(|| memo()) != new_value {
                set_memo(new_value);
            }
        }
    });

    // return memoized result
    let memo_result = move || Rc::new(Option::as_ref(&memo()).unwrap().clone());
    Rc::new(memo_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signals() {
        let (state, set_state) = create_signal(0);
        assert_eq!(*state(), 0);

        set_state(1);
        assert_eq!(*state(), 1);
    }

    #[test]
    fn signal_composition() {
        let (state, set_state) = create_signal(0);

        let double = || *state() * 2;

        assert_eq!(double(), 0);

        set_state(1);
        assert_eq!(double(), 2);
    }

    #[test]
    fn effects() {
        let (state, set_state) = create_signal(0);

        let (double, set_double) = create_signal(-1);

        create_effect({
            let set_double = set_double.clone();
            move || {
                set_double(*state() * 2);
            }
        });
        assert_eq!(*double(), 0); // calling create_effect should call the effect at least once

        set_state(1);
        assert_eq!(*double(), 2);
        set_state(2);
        assert_eq!(*double(), 4);
    }

    #[test]
    #[should_panic(expected = "cannot create cyclic dependency")]
    fn cyclic_effects_fail() {
        let (state, set_state) = create_signal(0);

        create_effect({
            let state = state.clone();
            let set_state = set_state.clone();
            move || {
                set_state(*state() + 1);
            }
        });

        set_state(1);
    }

    #[test]
    #[should_panic(expected = "cannot create cyclic dependency")]
    fn cyclic_effects_fail_2() {
        let (state, set_state) = create_signal(0);

        create_effect({
            let state = state.clone();
            let set_state = set_state.clone();
            move || {
                let value = *state();
                set_state(value + 1);
            }
        });

        set_state(1);
    }

    #[test]
    fn effect_should_subscribe_once() {
        let (state, set_state) = create_signal(0);

        let (counter, set_counter) = create_signal(0);
        create_effect({
            let counter = counter.clone();
            move || {
                set_counter(untracked(|| *counter()) + 1);

                // call state() twice but should subscribe once
                state();
                state();
            }
        });

        assert_eq!(*counter(), 1);

        set_state(1);
        assert_eq!(*counter(), 2);
    }

    #[test]
    fn memo() {
        let (state, set_state) = create_signal(0);

        let double = create_memo(move || *state() * 2);
        assert_eq!(*double(), 0);

        set_state(1);
        assert_eq!(*double(), 2);

        set_state(2);
        assert_eq!(*double(), 4);
    }

    #[test]
    /// Make sure value is memoized rather than executed on demand.
    fn memo_only_run_once() {
        let (state, set_state) = create_signal(0);

        let (counter, set_counter) = create_signal(0);
        let double = create_memo({
            let counter = counter.clone();
            move || {
                set_counter(untracked(|| *counter()) + 1);

                *state() * 2
            }
        });
        assert_eq!(*counter(), 1); // once for calculating initial derived state

        set_state(2);
        assert_eq!(*counter(), 2);
        assert_eq!(*double(), 4);
        assert_eq!(*counter(), 2); // should still be 2 after access
    }

    #[test]
    fn dependency_on_memo() {
        let (state, set_state) = create_signal(0);

        let double = create_memo(move || *state() * 2);

        let quadruple = create_memo(move || *double() * 2);

        assert_eq!(*quadruple(), 0);

        set_state(1);
        assert_eq!(*quadruple(), 4);
    }

    #[test]
    fn untracked_memo() {
        let (state, set_state) = create_signal(1);

        let double = create_memo(move || untracked(|| *state()) * 2);

        assert_eq!(*double(), 2);

        set_state(2);
        assert_eq!(*double(), 2); // double value should still be true because state() was inside untracked
    }

    #[test]
    fn selector() {
        let (state, set_state) = create_signal(0);

        let double = create_selector({
            let state = state.clone();
            move || *state() * 2
        });

        let (counter, set_counter) = create_signal(0);
        create_effect({
            let counter = counter.clone();
            let double = double.clone();
            move || {
                set_counter(untracked(|| *counter()) + 1);

                double();
            }
        });
        assert_eq!(*double(), 0);
        assert_eq!(*counter(), 1);

        set_state(0);
        assert_eq!(*double(), 0);
        assert_eq!(*counter(), 1); // calling set_state should not trigger the effect

        set_state(2);
        assert_eq!(*double(), 4);
        assert_eq!(*counter(), 2);
    }
}

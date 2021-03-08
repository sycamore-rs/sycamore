//! Reactive primitives.

use std::cell::Cell;
use std::cell::RefCell;
use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;

/// Returned by functions that provide a handle to access state.
pub struct StateHandle<T: 'static>(Rc<RefCell<SignalInner<T>>>);

impl<T: 'static> StateHandle<T> {
    /// Get the current value of the state.
    pub fn get(&self) -> Rc<T> {
        // if inside an effect, add this signal to dependency list
        DEPENDENCIES.with(|dependencies| {
            if dependencies.borrow().is_some() {
                let signal = self.0.clone();
                let handler = HANDLER.with(|handler| handler.borrow().as_ref().unwrap().clone());

                dependencies
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .push(Box::new(move || {
                        signal.borrow_mut().observe(handler.clone())
                    }));
            }
        });

        self.get_untracked()
    }

    /// Get the current value of the state, without tracking this as a dependency if inside a
    /// reactive context.
    ///
    /// # Example
    ///
    /// ```
    /// use maple_core::prelude::*;
    ///
    /// let state = Signal::new(1);
    ///
    /// let double = create_memo({
    ///     let state = state.clone();
    ///     move || *state.get_untracked() * 2
    /// });
    ///
    /// assert_eq!(*double.get(), 2);
    ///
    /// state.set(2);
    /// // double value should still be old value because state was untracked
    /// assert_eq!(*double.get(), 2);
    /// ```
    pub fn get_untracked(&self) -> Rc<T> {
        self.0.borrow().inner.clone()
    }
}

impl<T: 'static> Clone for StateHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// State that can be set.
pub struct Signal<T: 'static>(StateHandle<T>);

impl<T: 'static> Signal<T> {
    /// Creates a new signal.
    ///
    /// # Examples
    ///
    /// ```
    /// use maple_core::prelude::*;
    ///
    /// let state = Signal::new(0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// ```
    pub fn new(value: T) -> Self {
        Self(StateHandle(Rc::new(RefCell::new(SignalInner::new(value)))))
    }

    /// Set the current value of the state.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    pub fn set(&self, new_value: T) {
        match self.0 .0.try_borrow_mut() {
            Ok(mut signal) => signal.update(new_value),
            // If the signal is already borrowed, that means it is borrowed in the getter, thus creating a cyclic dependency.
            Err(_err) => panic!("cannot create cyclic dependency"),
        }
        self.0 .0.borrow().trigger_observers();
    }
}

impl<T: 'static> Deref for Signal<T> {
    type Target = StateHandle<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct SignalInner<T> {
    inner: Rc<T>,
    observers: Vec<Rc<Computation>>,
}

impl<T> SignalInner<T> {
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

/// Creates a memoized value from some signals. Also know as "derived stores".
pub fn create_memo<F, Out>(derived: F) -> StateHandle<Out>
where
    F: Fn() -> Out + 'static,
    Out: 'static,
{
    create_selector_with(derived, |_, _| false)
}

/// Creates a memoized value from some signals. Also know as "derived stores".
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is the same.
/// That is why the output of the function must implement [`PartialEq`].
///
/// To specify a custom comparison function, use [`create_selector_with`].
pub fn create_selector<F, Out>(derived: F) -> StateHandle<Out>
where
    F: Fn() -> Out + 'static,
    Out: PartialEq + 'static,
{
    create_selector_with(derived, PartialEq::eq)
}

/// Creates a memoized value from some signals. Also know as "derived stores".
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is the same.
///
/// It takes a comparison function to compare the old and new value, which returns `true` if they
/// are the same and `false` otherwise.
///
/// To use the type's [`PartialEq`] implementation instead of a custom function, use
/// [`create_selector`].
pub fn create_selector_with<F, Out, C>(derived: F, comparator: C) -> StateHandle<Out>
where
    F: Fn() -> Out + 'static,
    Out: 'static,
    C: Fn(&Out, &Out) -> bool + 'static,
{
    enum EffectState<T: 'static> {
        Uninitialized(NonNull<Option<StateHandle<T>>>),
        Initialized(Signal<T>),
        Temporary,
    }

    let mut handle = None;
    let state = Cell::new(EffectState::Uninitialized(NonNull::from(&mut handle)));

    create_effect(move || {
        let memo = match state.replace(EffectState::Temporary) {
            // We haven't been called yet, so initialize the signal.
            EffectState::Uninitialized(mut handle_ptr) => {
                let memo = Signal::new(derived());

                // SAFETY: This is the first time the function is called, and so this pointer is
                // still valid.
                *unsafe { handle_ptr.as_mut() } = Some((*memo).clone());

                memo
            }
            // We have been called; update the signal.
            EffectState::Initialized(memo) => {
                let new_value = derived();
                if !comparator(&memo.get_untracked(), &new_value) {
                    memo.set(new_value);
                }
                memo
            }
            EffectState::Temporary => unreachable!(),
        };
        state.replace(EffectState::Initialized(memo));
    });

    // By this time, the above closure will have been run once, setting the `handle` value.
    handle.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signals() {
        let state = Signal::new(0);
        assert_eq!(*state.get(), 0);

        state.set(1);
        assert_eq!(*state.get(), 1);
    }

    #[test]
    fn signal_composition() {
        let state = Signal::new(0);

        let double = || *state.get() * 2;

        assert_eq!(double(), 0);

        state.set(1);
        assert_eq!(double(), 2);
    }

    #[test]
    fn effects() {
        let state = Signal::new(0);

        let double = Signal::new(-1);

        create_effect({
            let state = state.clone();
            let double = double.clone();
            move || {
                double.set(*state.get() * 2);
            }
        });
        assert_eq!(*double.get(), 0); // calling create_effect should call the effect at least once

        state.set(1);
        assert_eq!(*double.get(), 2);
        state.set(2);
        assert_eq!(*double.get(), 4);
    }

    #[test]
    #[should_panic(expected = "cannot create cyclic dependency")]
    fn cyclic_effects_fail() {
        let state = Signal::new(0);

        create_effect({
            let state = state.clone();
            move || {
                state.set(*state.get() + 1);
            }
        });

        state.set(1);
    }

    #[test]
    #[should_panic(expected = "cannot create cyclic dependency")]
    fn cyclic_effects_fail_2() {
        let state = Signal::new(0);

        create_effect({
            let state = state.clone();
            move || {
                let value = *state.get();
                state.set(value + 1);
            }
        });

        state.set(1);
    }

    #[test]
    fn effect_should_subscribe_once() {
        let state = Signal::new(0);

        let counter = Signal::new(0);
        create_effect({
            let state = state.clone();
            let counter = counter.clone();
            move || {
                counter.set(*counter.get_untracked() + 1);

                // call state.get() twice but should subscribe once
                state.get();
                state.get();
            }
        });

        assert_eq!(*counter.get(), 1);

        state.set(1);
        assert_eq!(*counter.get(), 2);
    }

    #[test]
    fn memo() {
        let state = Signal::new(0);

        let double = create_memo({
            let state = state.clone();
            move || *state.get() * 2
        });
        assert_eq!(*double.get(), 0);

        state.set(1);
        assert_eq!(*double.get(), 2);

        state.set(2);
        assert_eq!(*double.get(), 4);
    }

    #[test]
    /// Make sure value is memoized rather than executed on demand.
    fn memo_only_run_once() {
        let state = Signal::new(0);

        let counter = Signal::new(0);
        let double = create_memo({
            let state = state.clone();
            let counter = counter.clone();
            move || {
                counter.set(*counter.get_untracked() + 1);

                *state.get() * 2
            }
        });
        assert_eq!(*counter.get(), 1); // once for calculating initial derived state

        state.set(2);
        assert_eq!(*counter.get(), 2);
        assert_eq!(*double.get(), 4);
        assert_eq!(*counter.get(), 2); // should still be 2 after access
    }

    #[test]
    fn dependency_on_memo() {
        let state = Signal::new(0);

        let double = create_memo({
            let state = state.clone();
            move || *state.get() * 2
        });

        let quadruple = create_memo(move || *double.get() * 2);

        assert_eq!(*quadruple.get(), 0);

        state.set(1);
        assert_eq!(*quadruple.get(), 4);
    }

    #[test]
    fn untracked_memo() {
        let state = Signal::new(1);

        let double = create_memo({
            let state = state.clone();
            move || *state.get_untracked() * 2
        });

        assert_eq!(*double.get(), 2);

        state.set(2);
        assert_eq!(*double.get(), 2); // double value should still be true because state.get() was inside untracked
    }

    #[test]
    fn selector() {
        let state = Signal::new(0);

        let double = create_selector({
            let state = state.clone();
            move || *state.get() * 2
        });

        let counter = Signal::new(0);
        create_effect({
            let counter = counter.clone();
            let double = double.clone();
            move || {
                counter.set(*counter.get_untracked() + 1);

                double.get();
            }
        });
        assert_eq!(*double.get(), 0);
        assert_eq!(*counter.get(), 1);

        state.set(0);
        assert_eq!(*double.get(), 0);
        assert_eq!(*counter.get(), 1); // calling set_state should not trigger the effect

        state.set(2);
        assert_eq!(*double.get(), 4);
        assert_eq!(*counter.get(), 2);
    }
}

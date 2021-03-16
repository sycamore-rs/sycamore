use std::mem;
use std::rc::Weak;

use super::*;

thread_local! {
    /// Context of the effect that is currently running. `None` if no effect is running.
    ///
    /// This is an array of callbacks that, when called, will add the a `Signal` to the `handle` in the argument.
    /// The callbacks return another callback which will unsubscribe the `handle` from the `Signal`.
    pub(super) static CONTEXTS: RefCell<Vec<Weak<RefCell<Option<Running>>>>> = RefCell::new(Vec::new());
    pub(super) static OWNER: RefCell<Option<Rc<RefCell<Owner>>>> = RefCell::new(None);
}

/// State of the current running effect.
/// When the state is dropped, all dependencies are removed (both links and backlinks).
pub(super) struct Running {
    pub(super) execute: Rc<dyn Fn()>,
    pub(super) dependencies: HashSet<Dependency>,
    _owner: Rc<RefCell<Owner>>,
}

impl Running {
    /// Clears the dependencies (both links and backlinks).
    /// Should be called when re-executing an effect to recreate all dependencies.
    fn clear_dependencies(&mut self) {
        for dependency in &self.dependencies {
            dependency
                .signal()
                .unsubscribe(&Callback(Rc::downgrade(&self.execute)));
        }
        self.dependencies.clear();
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        self.clear_dependencies();
    }
}

/// Owns the effects created in the current reactive scope.
#[derive(Default)]
pub struct Owner {
    effects: Vec<Rc<RefCell<Option<Running>>>>,
    cleanup: Vec<Box<dyn FnOnce()>>,
}

impl Owner {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn add_effect_state(&mut self, effect: Rc<RefCell<Option<Running>>>) {
        self.effects.push(effect);
    }

    pub(super) fn add_cleanup(&mut self, cleanup: Box<dyn FnOnce()>) {
        self.cleanup.push(cleanup);
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        for effect in &self.effects {
            effect.borrow_mut().as_mut().unwrap().clear_dependencies();
        }

        for cleanup in mem::take(&mut self.cleanup) {
            cleanup();
            panic!("abc")
        }
    }
}

#[derive(Clone)]
pub(super) struct Callback(pub(super) Weak<dyn Fn()>);

impl Callback {
    #[track_caller]
    #[must_use = "returned value must be manually called"]
    pub fn callback(&self) -> Rc<dyn Fn()> {
        self.try_callback().expect("callback is not valid anymore")
    }

    #[must_use = "returned value must be manually called"]
    pub fn try_callback(&self) -> Option<Rc<dyn Fn()>> {
        self.0.upgrade()
    }
}

impl Hash for Callback {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.callback()).hash(state);
    }
}

impl PartialEq for Callback {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq::<()>(
            Rc::as_ptr(&self.callback()).cast(),
            Rc::as_ptr(&other.callback()).cast(),
        )
    }
}
impl Eq for Callback {}

/// A [`Weak`] backlink to a [`Signal`] for any type `T`.
#[derive(Clone)]
pub(super) struct Dependency(pub(super) Weak<dyn AnySignalInner>);

impl Dependency {
    fn signal(&self) -> Rc<dyn AnySignalInner> {
        self.0.upgrade().expect("backlink should always be valid")
    }
}

impl Hash for Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.signal()).hash(state);
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq::<()>(
            Rc::as_ptr(&self.signal()).cast(),
            Rc::as_ptr(&other.signal()).cast(),
        )
    }
}
impl Eq for Dependency {}

/// Creates an effect on signals used inside the effect closure.
///
/// Unlike [`create_effect`], this will allow the closure to run different code upon first
/// execution, so it can return a value.
pub fn create_effect_initial<R: 'static + Clone>(
    initial: impl FnOnce() -> (Rc<dyn Fn()>, R) + 'static,
) -> R {
    let running: Rc<RefCell<Option<Running>>> = Rc::new(RefCell::new(None));

    let effect: RefCell<Option<Rc<dyn Fn()>>> = RefCell::new(None);
    let ret: Rc<RefCell<Option<R>>> = Rc::new(RefCell::new(None));

    let initial = RefCell::new(Some(initial));

    let execute: Rc<dyn Fn()> = Rc::new({
        let running = Rc::downgrade(&running);
        let ret = ret.clone();
        move || {
            CONTEXTS.with(|contexts| {
                let initial_context_size = contexts.borrow().len();

                running
                    .upgrade()
                    .unwrap()
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .clear_dependencies();

                contexts.borrow_mut().push(running.clone());

                if initial.borrow().is_some() {
                    let initial = initial.replace(None).unwrap();
                    let (effect_tmp, ret_tmp) = initial();
                    *effect.borrow_mut() = Some(effect_tmp);
                    *ret.borrow_mut() = Some(ret_tmp);
                } else {
                    // destroy old effects before new ones run
                    *running
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        ._owner
                        .borrow_mut() = Owner::new();

                    let effect = effect.clone();
                    let owner = create_root(move || {
                        effect.borrow().as_ref().unwrap()();
                    });
                    running
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        ._owner = owner;
                }

                // attach dependencies
                for dependency in &running
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .dependencies
                {
                    dependency.signal().subscribe(Callback(Rc::downgrade(
                        &running
                            .upgrade()
                            .unwrap()
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .execute,
                    )));
                }

                contexts.borrow_mut().pop();

                debug_assert_eq!(
                    initial_context_size,
                    contexts.borrow().len(),
                    "context size should not change"
                );
            });
        }
    });

    *running.borrow_mut() = Some(Running {
        execute: execute.clone(),
        dependencies: HashSet::new(),
        _owner: Rc::new(RefCell::new(Owner::new())),
    });
    debug_assert_eq!(
        Rc::strong_count(&running),
        1,
        "Running should be owned exclusively by owner"
    );

    OWNER.with(|owner| {
        if owner.borrow().is_some() {
            owner
                .borrow()
                .as_ref()
                .unwrap()
                .borrow_mut()
                .add_effect_state(running);
        } else {
            #[cfg(all(target_arch = "wasm32", debug_assertions))]
            web_sys::console::warn_1(
                &"Effects created outside of a reactive root will never get disposed.".into(),
            );
            #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
            eprintln!(
                "WARNING: Effects created outside of a reactive root will never get dropped."
            );
            Rc::into_raw(running); // leak running
        }
    });

    execute();

    let ret = ret.borrow();
    ret.as_ref().unwrap().clone()
}

/// Creates an effect on signals used inside the effect closure.
pub fn create_effect<F>(effect: F)
where
    F: Fn() + 'static,
{
    create_effect_initial(move || {
        effect();
        (Rc::new(effect), ())
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
    let derived = Rc::new(derived);
    let comparator = Rc::new(comparator);

    create_effect_initial(move || {
        let memo = Signal::new(derived());

        let effect = Rc::new({
            let memo = memo.clone();
            let derived = derived.clone();
            move || {
                let new_value = derived();
                if !comparator(&memo.get_untracked(), &new_value) {
                    memo.set(new_value);
                }
            }
        });

        (effect, memo.into_handle())
    })
}

/// Adds a callback function to the current reactive scope's cleanup.
/// 
/// # Example
/// ```
/// use maple_core::prelude::*;
/// 
/// let cleanup_called = Signal::new(false);
/// 
/// let owner = create_root(cloned!((cleanup_called) => move || {
///     on_cleanup(move || {
///         cleanup_called.set(true);
///     })
/// }));
/// 
/// assert_eq!(*cleanup_called.get(), false);
/// 
/// drop(owner);
/// assert_eq!(*cleanup_called.get(), true);
/// ```
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    OWNER.with(|owner| {
        if owner.borrow().is_some() {
            owner
                .borrow()
                .as_ref()
                .unwrap()
                .borrow_mut()
                .add_cleanup(Box::new(f));
        } else {
            #[cfg(all(target_arch = "wasm32", debug_assertions))]
            web_sys::console::warn_1(
                &"Cleanup callbacks created outside of a reactive root will never run.".into(),
            );
            #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
            eprintln!(
                "WARNING: Cleanup callbacks created outside of a reactive root will never run."
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloned;

    #[test]
    fn effects() {
        let state = Signal::new(0);

        let double = Signal::new(-1);

        create_effect(cloned!((state, double) => move || {
            double.set(*state.get() * 2);
        }));
        assert_eq!(*double.get(), 0); // calling create_effect should call the effect at least once

        state.set(1);
        assert_eq!(*double.get(), 2);
        state.set(2);
        assert_eq!(*double.get(), 4);
    }

    // FIXME: cycle detection is currently broken
    #[test]
    #[ignore]
    #[should_panic(expected = "cannot create cyclic dependency")]
    fn cyclic_effects_fail() {
        let state = Signal::new(0);

        create_effect(cloned!((state) => move || {
            state.set(*state.get() + 1);
        }));

        state.set(1);
    }

    #[test]
    #[ignore]
    #[should_panic(expected = "cannot create cyclic dependency")]
    fn cyclic_effects_fail_2() {
        let state = Signal::new(0);

        create_effect(cloned!((state) => move || {
            let value = *state.get();
            state.set(value + 1);
        }));

        state.set(1);
    }

    #[test]
    fn effect_should_subscribe_once() {
        let state = Signal::new(0);

        let counter = Signal::new(0);
        create_effect(cloned!((state, counter) => move || {
            counter.set(*counter.get_untracked() + 1);

            // call state.get() twice but should subscribe once
            state.get();
            state.get();
        }));

        assert_eq!(*counter.get(), 1);

        state.set(1);
        assert_eq!(*counter.get(), 2);
    }

    #[test]
    fn effect_should_recreate_dependencies() {
        let condition = Signal::new(true);

        let state1 = Signal::new(0);
        let state2 = Signal::new(1);

        let counter = Signal::new(0);
        create_effect(cloned!((condition, state1, state2, counter) => move || {
            counter.set(*counter.get_untracked() + 1);

            if *condition.get() {
                state1.get();
            } else {
                state2.get();
            }
        }));

        assert_eq!(*counter.get(), 1);

        state1.set(1);
        assert_eq!(*counter.get(), 2);

        state2.set(1);
        assert_eq!(*counter.get(), 2); // not tracked

        condition.set(false);
        assert_eq!(*counter.get(), 3);

        state1.set(2);
        assert_eq!(*counter.get(), 3); // not tracked

        state2.set(2);
        assert_eq!(*counter.get(), 4); // tracked after condition.set
    }

    #[test]
    fn nested_effects_should_recreate_inner() {
        let counter = Signal::new(0);

        let trigger = Signal::new(());

        create_effect(cloned!((trigger, counter) => move || {
            trigger.get(); // subscribe to trigger

            create_effect(cloned!((counter) => move || {
                counter.set(*counter.get_untracked() + 1);
            }));
        }));

        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 2); // old inner effect should be destroyed and thus not executed
    }

    #[test]
    fn destroy_effects_on_owner_drop() {
        let counter = Signal::new(0);

        let trigger = Signal::new(());

        let owner = create_root(cloned!((trigger, counter) => move || {
            create_effect(move || {
                trigger.get(); // subscribe to trigger
                counter.set(*counter.get_untracked() + 1);
            });
        }));

        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 2);

        drop(owner);
        trigger.set(());
        assert_eq!(*counter.get(), 2); // inner effect should be destroyed and thus not executed
    }

    #[test]
    fn memo() {
        let state = Signal::new(0);

        let double = create_memo(cloned!((state) => move || *state.get() * 2));
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
        let double = create_memo(cloned!((state, counter) => move || {
            counter.set(*counter.get_untracked() + 1);

            *state.get() * 2
        }));
        assert_eq!(*counter.get(), 1); // once for calculating initial derived state

        state.set(2);
        assert_eq!(*counter.get(), 2);
        assert_eq!(*double.get(), 4);
        assert_eq!(*counter.get(), 2); // should still be 2 after access
    }

    #[test]
    fn dependency_on_memo() {
        let state = Signal::new(0);

        let double = create_memo(cloned!((state) => move || *state.get() * 2));

        let quadruple = create_memo(move || *double.get() * 2);

        assert_eq!(*quadruple.get(), 0);

        state.set(1);
        assert_eq!(*quadruple.get(), 4);
    }

    #[test]
    fn untracked_memo() {
        let state = Signal::new(1);

        let double = create_memo(cloned!((state) => move || *state.get_untracked() * 2));

        assert_eq!(*double.get(), 2);

        state.set(2);
        assert_eq!(*double.get(), 2); // double value should still be true because state.get() was inside untracked
    }

    #[test]
    fn selector() {
        let state = Signal::new(0);

        let double = create_selector(cloned!((state) => move || *state.get() * 2));

        let counter = Signal::new(0);
        create_effect(cloned!((counter, double) => move || {
            counter.set(*counter.get_untracked() + 1);

            double.get();
        }));
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

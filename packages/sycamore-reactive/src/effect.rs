//! Side effects and derived signals.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::mem;
use std::rc::{Rc, Weak};

use crate::scope::{create_root, current_scope, ReactiveScope, SCOPE_STACK};
use crate::signal::{create_signal, ReadSignal, SignalId, WriteSignal};

thread_local! {
    /// The current effect listener or `None`.
    pub(crate) static CURRENT_LISTENER: RefCell<Option<Listener>> = RefCell::new(None);
}

pub(crate) struct EffectState {
    pub callback: Rc<RefCell<dyn FnMut()>>,
    dependencies: HashSet<SignalId>,
    scope: Option<ReactiveScope>,
}

pub(crate) struct Listener(Rc<RefCell<Option<EffectState>>>);

impl Listener {
    /// Add a dependency to the effect.
    pub fn add_dependency(&self, signal: SignalId) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .dependencies
            .insert(signal);
    }

    /// Clears the dependencies (both links and backlinks).
    /// Should be called when re-executing an effect to recreate all dependencies.
    fn clear_dependencies(&self) {
        for dependency in &self.0.borrow().as_ref().unwrap().dependencies {
            dependency.get_mut(|data| {
                if let Some(data) = data {
                    data.unsubscribe(Rc::as_ptr(&self.0));
                }
            });
        }
        self.0.borrow_mut().as_mut().unwrap().dependencies.clear();
    }
}

pub(crate) type EffectStatePtr = *const RefCell<Option<EffectState>>;

/// Creates a new effect. Any signals that are accessed inside the effect closure are added as
/// dependencies. When a dependency is updated, the effect is re-executed.
///
/// The effect closure is executed in a new reactive scope which is recreated on every re-execution.
/// This means that signals created within the effect might not be valid outside the effect. This
/// also means that inner effects created within this effect will be recreated.
///
/// # Panics
/// This function will `panic!()` if it is called outside of a reactive scope.
/// # Example
/// ```
/// # use sycamore_reactive::effect::create_effect;
/// # use sycamore_reactive::scope::create_root;
/// # use sycamore_reactive::signal::create_signal;
///
/// # let _ = create_root(|| {
/// let (state, set_state) = create_signal(0);
///
/// create_effect(move || {
///     println!("State changed. New state value = {}", state.get());
/// }); // Prints "State changed. New state value = 0"
///
/// set_state.set(1); // Prints "State changed. New state value = 1"
/// # });
/// ```
pub fn create_effect(mut f: impl FnMut() + 'static) {
    let effect_state = Rc::new(RefCell::new(None));

    // Callback for when the effect's dependencies are triggered.
    let callback: Rc<RefCell<dyn FnMut()>> = Rc::new(RefCell::new({
        let effect_state = Rc::downgrade(&effect_state);
        move || {
            CURRENT_LISTENER.with(|listener| {
                // Upgrade effect state now to ensure that it is valid for the entire effect.
                let effect_state = Weak::upgrade(&effect_state).unwrap();
                // Create new listener.
                let new_listener = Listener(Rc::clone(&effect_state));
                // Recreate effect dependencies each time effect is called.
                new_listener.clear_dependencies();
                // Swap in the new listener.
                let old_listener = mem::replace(&mut *listener.borrow_mut(), Some(new_listener));

                // Destroy old effects before new ones run.

                // We want to destroy the old scope before creating the new one, so that
                // cleanup functions will be run before the effect
                // closure is called again.
                let _: Option<ReactiveScope> =
                    mem::take(&mut effect_state.borrow_mut().as_mut().unwrap().scope);

                // Run the effect in a new scope.
                let scope = create_root(|| {
                    f();
                });
                effect_state.borrow_mut().as_mut().unwrap().scope = Some(scope);

                // Attach new dependencies.
                let effect_state_ref = effect_state.borrow();
                let effect_state_ref = effect_state_ref.as_ref().unwrap();
                for dependency in &effect_state_ref.dependencies {
                    dependency.get_mut(|data| {
                        if let Some(data) = data {
                            // Signal might have already been destroyed inside the effect.
                            data.subscribe(Rc::downgrade(&effect_state))
                        }
                    })
                }

                // Restore old listener.
                mem::replace(&mut *listener.borrow_mut(), old_listener).unwrap();
            });
        }
    }));

    *effect_state.borrow_mut() = Some(EffectState {
        callback: Rc::clone(&callback),
        dependencies: HashSet::new(),
        scope: None,
    });
    // Check that effect_state is only has 1 strong reference. This is to ensure that it is owned
    // exclusively by the ReactiveScope.
    debug_assert_eq!(Rc::strong_count(&effect_state), 1);
    // Check that there are no outstanding borrows.
    debug_assert!(RefCell::try_borrow_mut(&effect_state).is_ok());

    // Effect always calls the callback once.
    callback.borrow_mut()();

    // Move effect to current scope.
    SCOPE_STACK.with(|scope_stack| {
        if let Some(scope) = scope_stack.borrow().last() {
            scope.add_effect_state(effect_state);
        } else {
            panic!("create_effect must be used inside a reactive scope")
        }
    });
}

/// Creates a memoized value from some signals. Also know as "derived stores".
///
/// # Example
/// ```
/// # use sycamore_reactive::effect::create_memo;
/// # use sycamore_reactive::scope::create_root;
/// # use sycamore_reactive::signal::create_signal;
///
/// # let _ = create_root(|| {
/// let (state, set_state) = create_signal(0);
///
/// let double = create_memo(move || *state.get() * 2);
/// assert_eq!(*double.get(), 0);
///
/// set_state.set(1);
/// assert_eq!(*double.get(), 2);
/// # });
/// ```
///
/// # Panics
/// This function will `panic!()` if it is used outside of a reactive scope.
pub fn create_memo<F, T>(derived: F) -> ReadSignal<T>
where
    F: FnMut() -> T + 'static,
    T: 'static,
{
    create_selector_with(derived, |_, _| false)
}

/// Creates a memoized value from some signals. Also know as "derived stores".
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is
/// the same. That is why the output of the function must implement [`PartialEq`].
///
/// To specify a custom comparison function, use [`create_selector_with`].
///
/// # Panics
/// This function will `panic!()` if it is used outside of a reactive scope.
#[track_caller]
pub fn create_selector<F, T>(derived: F) -> ReadSignal<T>
where
    F: FnMut() -> T + 'static,
    T: PartialEq + 'static,
{
    create_selector_with(derived, PartialEq::eq)
}

/// Creates a memoized value from some signals. Also know as "derived stores".
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is
/// the same.
///
/// It takes a comparison function to compare the old and new value, which returns `true` if they
/// are the same and `false` otherwise.
///
/// To use the type's [`PartialEq`] implementation instead of a custom function, use
/// [`create_selector`].
///
/// # Panics
/// This function will `panic!()` if it is used outside of a reactive scope.
#[track_caller]
pub fn create_selector_with<F, T, C>(mut derived: F, comparator: C) -> ReadSignal<T>
where
    F: FnMut() -> T + 'static,
    T: 'static,
    C: Fn(&T, &T) -> bool + 'static,
{
    let memo = Rc::new(Cell::new(None::<(ReadSignal<T>, WriteSignal<T>)>));

    let mut scope =
        Some(current_scope().expect("create_signal must be used inside a ReactiveScope"));

    create_effect({
        let memo = Rc::clone(&memo);
        move || {
            let new_value = derived();
            if let Some((memo, set_memo)) = memo.get() {
                if !comparator(&memo.get_untracked(), &new_value) {
                    set_memo.set(new_value);
                }
            } else {
                // This branch will only be executed once.
                scope.as_ref().unwrap().extend(|| {
                    // We want the signal to live as long as the outer scope instead of the effect
                    // scope.
                    memo.set(Some(create_signal(new_value)));
                });
                // Do not hold a reference to the scope. This prevents a memory leak because the
                // scope also owns the effect and thereby the effect closure.
                drop(scope.take());
            }
            debug_assert!(memo.get().is_some());
        }
    });

    memo.get().unwrap().0
}

/// Run the passed closure inside an untracked dependency scope.
///
/// This does **NOT** create a new [`ReactiveScope`].
///
/// See also [`ReadSignal::get_untracked`].
///
/// # Example
///
/// ```
/// # use sycamore_reactive::effect::{create_memo, untrack};
/// # use sycamore_reactive::scope::create_root;
/// # use sycamore_reactive::signal::create_signal;
///
/// # let _ = create_root(|| {
/// let (state, set_state) = create_signal(1);
///
/// let double = create_memo(move || untrack(|| *state.get() * 2));
///
/// assert_eq!(*double.get(), 2);
///
/// set_state.set(2);
/// // double value should still be old value because state was untracked
/// assert_eq!(*double.get(), 2);
/// # });
/// ```
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    CURRENT_LISTENER.with(|current_listener| {
        let scope = mem::take(&mut *current_listener.borrow_mut());
        let ret = f();
        *current_listener.borrow_mut() = scope;
        ret
    })
}

#[cfg(test)]
mod tests {
    use crate::scope::on_cleanup;
    use crate::signal::create_signal;

    use super::*;

    #[test]
    fn effects() {
        let _ = create_root(|| {
            let (state, set_state) = create_signal(0);

            let (double, set_double) = create_signal(1);

            create_effect(move || {
                set_double.set(*state.get() * 2);
            });
            assert_eq!(*double.get(), 0); // calling create_effect should call the effect at least once

            set_state.set(1);
            assert_eq!(*double.get(), 2);
            set_state.set(2);
            assert_eq!(*double.get(), 4);
        });
    }

    #[test]
    fn effect_should_subscribe_once() {
        let _ = create_root(|| {
            let (state, set_state) = create_signal(0);

            let (counter, set_counter) = create_signal(0);
            create_effect(move || {
                set_counter.set(*counter.get_untracked() + 1);

                // call state.get() twice but should subscribe once
                state.get();
                state.get();
            });

            assert_eq!(*counter.get(), 1);

            set_state.set(1);
            assert_eq!(*counter.get(), 2);
        });
    }

    #[test]
    fn effect_should_recreate_dependencies() {
        let _ = create_root(|| {
            let (condition, set_condition) = create_signal(true);

            let (state1, set_state1) = create_signal(0);
            let (state2, set_state2) = create_signal(1);

            let (counter, set_counter) = create_signal(0);
            create_effect(move || {
                set_counter.set(*counter.get_untracked() + 1);

                if *condition.get() {
                    state1.get();
                } else {
                    state2.get();
                }
            });

            assert_eq!(*counter.get(), 1);

            set_state1.set(1);
            assert_eq!(*counter.get(), 2);

            set_state2.set(1);
            assert_eq!(*counter.get(), 2); // not tracked

            set_condition.set(false);
            assert_eq!(*counter.get(), 3);

            set_state1.set(2);
            assert_eq!(*counter.get(), 3); // not tracked

            set_state2.set(2);
            assert_eq!(*counter.get(), 4); // tracked after condition.set
        });
    }

    #[test]
    fn nested_effects_should_recreate_inner() {
        let _ = create_root(|| {
            let (counter, set_counter) = create_signal(0);

            let (trigger, set_trigger) = create_signal(());

            create_effect(move || {
                trigger.get(); // subscribe to trigger

                create_effect(move || {
                    set_counter.set(*counter.get_untracked() + 1);
                });
            });

            assert_eq!(*counter.get(), 1);

            set_trigger.set(());
            assert_eq!(*counter.get(), 2); // old inner effect should be destroyed and thus not
                                           // executed
        });
    }

    #[test]
    fn nested_effects_trigger_outer_effect() {
        let _ = create_root(|| {
            let (trigger, set_trigger) = create_signal(());

            let (outer_counter, set_outer_counter) = create_signal(0);
            let (inner_counter, set_inner_counter) = create_signal(0);
            let (inner_cleanup_counter, set_inner_cleanup_counter) = create_signal(0);

            create_effect(move || {
                trigger.get(); // subscribe to trigger
                set_outer_counter.set(*outer_counter.get_untracked() + 1);

                create_effect(move || {
                    set_trigger.set(()); // update trigger which should recreate the outer effect
                    set_inner_counter.set(*inner_counter.get_untracked() + 1);

                    on_cleanup(move || {
                        set_inner_cleanup_counter.set(*inner_cleanup_counter.get_untracked() + 1);
                    });
                });
            });

            assert_eq!(*outer_counter.get(), 1);
            assert_eq!(*inner_counter.get(), 1);
            assert_eq!(*inner_cleanup_counter.get(), 0);

            set_trigger.set(());

            assert_eq!(*outer_counter.get(), 2);
            assert_eq!(*inner_counter.get(), 2);
            assert_eq!(*inner_cleanup_counter.get(), 1);
        });
    }

    #[test]
    fn create_nested_effect_from_outside() {
        let _ = create_root(|| {
            let (trigger, set_trigger) = create_signal(());
            let (outer_counter, set_outer_counter) = create_signal(0);
            let (inner_counter, set_inner_counter) = create_signal(0);

            let (inner_effect, set_inner_effect) = create_signal(None::<Box<dyn Fn()>>);

            create_effect(move || {
                trigger.get(); // subscribe to trigger
                set_outer_counter.set(*outer_counter.get_untracked() + 1);

                if inner_effect.get_untracked().is_none() {
                    set_inner_effect.set(Some(Box::new(move || {
                        set_inner_counter.set(*inner_counter.get_untracked() + 1);
                    })));
                }
            });

            create_effect(move || (*inner_effect.get()).as_ref().unwrap()());

            assert_eq!(*outer_counter.get(), 1);
            assert_eq!(*inner_counter.get(), 1);

            set_trigger.set(());
            assert_eq!(*outer_counter.get(), 2);
            assert_eq!(*inner_counter.get(), 1);
        });
    }

    #[test]
    fn outer_effects_rerun_first() {
        let _ = create_root(|| {
            let (trigger, set_trigger) = create_signal(());

            let (outer_counter, set_outer_counter) = create_signal(0);
            let (inner_counter, set_inner_counter) = create_signal(0);

            create_effect(move || {
                trigger.get(); // subscribe to trigger
                set_outer_counter.set(*outer_counter.get_untracked() + 1);

                create_effect(move || {
                    trigger.get(); // subscribe to trigger
                    set_inner_counter.set(*inner_counter.get_untracked() + 1);
                });
            });

            assert_eq!(*outer_counter.get(), 1);
            assert_eq!(*inner_counter.get(), 1);

            set_trigger.set(());

            assert_eq!(*outer_counter.get(), 2);
            assert_eq!(*inner_counter.get(), 2);
        });
    }

    #[test]
    fn destroy_effects_on_scope_drop() {
        let _ = create_root(|| {
            let (counter, set_counter) = create_signal(0);

            let (trigger, set_trigger) = create_signal(());

            let scope = create_root(move || {
                create_effect(move || {
                    trigger.get(); // subscribe to trigger
                    set_counter.set(*counter.get_untracked() + 1);
                });
            });

            assert_eq!(*counter.get(), 1);

            set_trigger.set(());
            assert_eq!(*counter.get(), 2);

            drop(scope);
            set_trigger.set(());
            assert_eq!(*counter.get(), 2); // inner effect should be destroyed and thus not executed
        });
    }

    #[test]
    fn memo() {
        let _ = create_root(|| {
            let (state, set_state) = create_signal(0);

            let double = create_memo(move || *state.get() * 2);
            assert_eq!(*double.get(), 0);

            set_state.set(1);
            assert_eq!(*double.get(), 2);

            set_state.set(2);
            assert_eq!(*double.get(), 4);
        });
    }

    #[test]
    /// Make sure value is memoized rather than executed on demand.
    fn memo_only_run_once() {
        let _ = create_root(|| {
            let (state, set_state) = create_signal(0);

            let (counter, set_counter) = create_signal(0);
            let double = create_memo(move || {
                set_counter.set(*counter.get_untracked() + 1);

                *state.get() * 2
            });
            assert_eq!(*counter.get(), 1); // once for calculating initial derived state

            set_state.set(2);
            assert_eq!(*counter.get(), 2);
            assert_eq!(*double.get(), 4);
            assert_eq!(*counter.get(), 2); // should still be 2 after access
        });
    }

    #[test]
    fn dependency_on_memo() {
        let _ = create_root(|| {
            let (state, set_state) = create_signal(0);

            let double = create_memo(move || *state.get() * 2);

            let quadruple = create_memo(move || *double.get() * 2);

            assert_eq!(*quadruple.get(), 0);

            set_state.set(1);
            assert_eq!(*quadruple.get(), 4);
        });
    }

    #[test]
    fn untracked_memo() {
        let _ = create_root(|| {
            let (state, set_state) = create_signal(1);

            let double = create_memo(move || *state.get_untracked() * 2);

            assert_eq!(*double.get(), 2);

            set_state.set(2);
            assert_eq!(*double.get(), 2); // double value should still be true because state.get()
                                          // was
                                          // inside untracked
        });
    }

    #[test]
    fn selector() {
        let _ = create_root(|| {
            let (state, set_state) = create_signal(0);

            let double = create_selector(move || *state.get() * 2);

            let (counter, set_counter) = create_signal(0);
            create_effect(move || {
                set_counter.set(*counter.get_untracked() + 1);

                double.get();
            });
            assert_eq!(*double.get(), 0);
            assert_eq!(*counter.get(), 1);

            set_state.set(0);
            assert_eq!(*double.get(), 0);
            assert_eq!(*counter.get(), 1); // calling set_state should not trigger the effect

            set_state.set(2);
            assert_eq!(*double.get(), 4);
            assert_eq!(*counter.get(), 2);
        });
    }

    #[test]
    fn cleanup() {
        let _ = create_root(|| {
            let (cleanup_called, set_cleanup_called) = create_signal(false);
            let scope = create_root(move || {
                on_cleanup(move || {
                    set_cleanup_called.set(true);
                });
            });

            assert!(!*cleanup_called.get());

            drop(scope);
            assert!(*cleanup_called.get());
        });
    }

    #[test]
    fn cleanup_in_effect() {
        let _ = create_root(|| {
            let (trigger, set_trigger) = create_signal(());

            let (counter, set_counter) = create_signal(0);

            create_effect(move || {
                trigger.get(); // subscribe to trigger

                on_cleanup(move || {
                    set_counter.set(*counter.get() + 1);
                });
            });

            assert_eq!(*counter.get(), 0);

            set_trigger.set(());
            assert_eq!(*counter.get(), 1);

            set_trigger.set(());
            assert_eq!(*counter.get(), 2);
        });
    }

    #[test]
    fn cleanup_is_untracked() {
        let _ = create_root(|| {
            let (trigger, set_trigger) = create_signal(());

            let (counter, set_counter) = create_signal(0);

            create_effect(move || {
                set_counter.set(*counter.get_untracked() + 1);

                on_cleanup(move || {
                    trigger.get(); // do not subscribe to trigger
                });
            });

            assert_eq!(*counter.get(), 1);

            set_trigger.set(());
            assert_eq!(*counter.get(), 1);
        });
    }
}

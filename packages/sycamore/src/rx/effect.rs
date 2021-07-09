use std::any::Any;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::rc::Weak;

use super::*;

thread_local! {
    /// Context of the effect that is currently running. `None` if no effect is running.
    ///
    /// This is an array of callbacks that, when called, will add the a `Signal` to the `handle` in the argument.
    /// The callbacks return another callback which will unsubscribe the `handle` from the `Signal`.
    pub(super) static CONTEXTS: RefCell<Vec<Weak<RefCell<Option<Running>>>>> = RefCell::new(Vec::new());
    pub(super) static SCOPE: RefCell<Option<ReactiveScope>> = RefCell::new(None);
}

/// State of the current running effect.
/// When the state is dropped, all dependencies are removed (both links and backlinks).
pub(super) struct Running {
    /// Callback to run when the effect is recreated.
    pub(super) execute: Rc<RefCell<dyn FnMut()>>,
    /// A list of dependencies which trigger the effect.
    pub(super) dependencies: HashSet<Dependency>,
    /// The reactive scope owns all effects created within it.
    scope: ReactiveScope,
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
/// The effects are dropped and the cleanup callbacks are called when the [`ReactiveScope`] is
/// dropped.
#[derive(Default)]
pub struct ReactiveScope {
    /// Effects created in this scope.
    effects: Vec<Rc<RefCell<Option<Running>>>>,
    /// Callbacks to call when the scope is dropped.
    cleanup: Vec<Box<dyn FnOnce()>>,
}

impl ReactiveScope {
    /// Create a new empty [`ReactiveScope`].
    ///
    /// This should be rarely used and only serve as a placeholder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an effect that is owned by this [`ReactiveScope`].
    pub(super) fn add_effect_state(&mut self, effect: Rc<RefCell<Option<Running>>>) {
        self.effects.push(effect);
    }

    /// Add a cleanup callback that will be called when the [`ReactiveScope`] is dropped.
    pub(super) fn add_cleanup(&mut self, cleanup: Box<dyn FnOnce()>) {
        self.cleanup.push(cleanup);
    }
}

impl Drop for ReactiveScope {
    fn drop(&mut self) {
        for effect in &self.effects {
            effect.borrow_mut().as_mut().unwrap().clear_dependencies();
        }

        for cleanup in mem::take(&mut self.cleanup) {
            untrack(cleanup);
        }
    }
}

#[derive(Clone)]
pub(super) struct Callback(pub(super) Weak<RefCell<dyn FnMut()>>);

impl Callback {
    #[track_caller]
    #[must_use = "returned value must be manually called"]
    pub fn callback(&self) -> Rc<RefCell<dyn FnMut()>> {
        self.try_callback().expect("callback is not valid anymore")
    }

    #[must_use = "returned value must be manually called"]
    pub fn try_callback(&self) -> Option<Rc<RefCell<dyn FnMut()>>> {
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

/// A strong backlink to a [`Signal`] for any type `T`.
#[derive(Clone)]
pub(super) struct Dependency(pub(super) Rc<dyn AnySignalInner>);

impl Dependency {
    fn signal(&self) -> Rc<dyn AnySignalInner> {
        Rc::clone(&self.0)
    }
}

impl Hash for Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq::<()>(Rc::as_ptr(&self.0).cast(), Rc::as_ptr(&other.0).cast())
    }
}
impl Eq for Dependency {}

/// Creates an effect on signals used inside the effect closure.
///
/// Unlike [`create_effect`], this will allow the closure to run different code upon first
/// execution, so it can return a value.
pub fn create_effect_initial<R: 'static>(
    initial: impl FnOnce() -> (Box<dyn FnMut()>, R) + 'static,
) -> R {
    type InitialFn = dyn FnOnce() -> (Box<dyn FnMut()>, Box<dyn Any>);

    /// Internal implementation: use dynamic dispatch to reduce code bloat.
    fn internal(initial: Box<InitialFn>) -> Box<dyn Any> {
        let running: Rc<RefCell<Option<Running>>> = Rc::new(RefCell::new(None));

        let mut effect: Option<Box<dyn FnMut()>> = None;
        let ret: Rc<RefCell<Option<Box<dyn Any>>>> = Rc::new(RefCell::new(None));

        let mut initial = Some(initial);

        // Callback for when the effect's dependencies are triggered.
        let execute: Rc<RefCell<dyn FnMut()>> = Rc::new(RefCell::new({
            let running = Rc::downgrade(&running);
            let ret = Rc::downgrade(&ret);
            move || {
                CONTEXTS.with(|contexts| {
                    // Record initial context size to verify that it is the same after.
                    let initial_context_size = contexts.borrow().len();

                    // Upgrade running now to make sure running is valid for the whole duration of
                    // the effect.
                    let running = running.upgrade().unwrap();

                    // Recreate effect dependencies each time effect is called.
                    running.borrow_mut().as_mut().unwrap().clear_dependencies();

                    // Push new reactive scope.
                    contexts.borrow_mut().push(Rc::downgrade(&running));

                    if let Some(initial) = initial.take() {
                        // Call initial callback.
                        let ret = Weak::upgrade(&ret).unwrap();
                        let scope = create_root(|| {
                            // Run initial effect closure.
                            let (effect_tmp, ret_tmp) = initial();
                            effect = Some(effect_tmp);
                            *ret.borrow_mut() = Some(ret_tmp);
                        });
                        running.borrow_mut().as_mut().unwrap().scope = scope;
                    } else {
                        // Destroy old effects before new ones run.

                        // We want to destroy the old scope before creating the new one, so that
                        // cleanup functions will be run before the effect
                        // closure is called again.
                        mem::take(&mut running.borrow_mut().as_mut().unwrap().scope);

                        // Run effect closure.
                        let new_scope = create_root(|| {
                            effect.as_mut().unwrap()();
                        });
                        running.borrow_mut().as_mut().unwrap().scope = new_scope;
                    }

                    // Attach new dependencies.
                    for dependency in &running.borrow().as_ref().unwrap().dependencies {
                        dependency.signal().subscribe(Callback(Rc::downgrade(
                            // Reference the same closure we are in right now.
                            // When the dependency changes, this closure will be called again.
                            &running.borrow().as_ref().unwrap().execute,
                        )));
                    }

                    // Remove reactive context.
                    contexts.borrow_mut().pop();

                    debug_assert_eq!(
                        initial_context_size,
                        contexts.borrow().len(),
                        "context size should not change before and after create_effect_initial"
                    );
                });
            }
        }));

        *running.borrow_mut() = Some(Running {
            execute: Rc::clone(&execute),
            dependencies: HashSet::new(),
            scope: ReactiveScope::new(),
        });
        debug_assert_eq!(
            Rc::strong_count(&running),
            1,
            "Running should be owned exclusively by ReactiveScope"
        );

        SCOPE.with(|scope| {
            if scope.borrow().is_some() {
                scope
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .add_effect_state(running);
            } else {
                thread_local! {
                    static GLOBAL_SCOPE: RefCell<ReactiveScope> = RefCell::new(ReactiveScope::new());
                }
                GLOBAL_SCOPE
                    .with(|global_scope| global_scope.borrow_mut().add_effect_state(running));
            }
        });

        execute.borrow_mut()();

        let ret = Rc::try_unwrap(ret).unwrap(); // ret should only have 1 strong reference
        ret.into_inner().unwrap()
    }

    let ret = internal(Box::new(|| {
        let (effect, ret) = initial();
        (effect, Box::new(ret))
    }));

    *ret.downcast::<R>().unwrap()
}

/// Creates an effect on signals used inside the effect closure.
///
/// # Example
/// ```
/// use sycamore::prelude::*;
///
/// let state = Signal::new(0);
///
/// create_effect(cloned!((state) => move || {
///     println!("State changed. New state value = {}", state.get());
/// })); // Prints "State changed. New state value = 0"
///
/// state.set(1); // Prints "State changed. New state value = 1"
/// ```
pub fn create_effect<F>(mut effect: F)
where
    F: FnMut() + 'static,
{
    create_effect_initial(move || {
        effect();
        (Box::new(effect), ())
    });
}

/// Creates a memoized value from some signals. Also know as "derived stores".
///
/// # Example
/// ```
/// use sycamore::prelude::*;
///
/// let state = Signal::new(0);
///
/// let double = create_memo(cloned!((state) => move || *state.get() * 2));
/// assert_eq!(*double.get(), 0);
///
/// state.set(1);
/// assert_eq!(*double.get(), 2);
/// ```
pub fn create_memo<F, Out>(derived: F) -> StateHandle<Out>
where
    F: FnMut() -> Out + 'static,
    Out: 'static,
{
    create_selector_with(derived, |_, _| false)
}

/// Creates a memoized value from some signals. Also know as "derived stores".
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is
/// the same. That is why the output of the function must implement [`PartialEq`].
///
/// To specify a custom comparison function, use [`create_selector_with`].
pub fn create_selector<F, Out>(derived: F) -> StateHandle<Out>
where
    F: FnMut() -> Out + 'static,
    Out: PartialEq + 'static,
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
pub fn create_selector_with<F, Out, C>(derived: F, comparator: C) -> StateHandle<Out>
where
    F: FnMut() -> Out + 'static,
    Out: 'static,
    C: Fn(&Out, &Out) -> bool + 'static,
{
    let derived = Rc::new(RefCell::new(derived));

    create_effect_initial(move || {
        let memo = Signal::new(derived.borrow_mut()());

        let effect = {
            let memo = memo.clone();
            let derived = Rc::clone(&derived);
            move || {
                let new_value = derived.borrow_mut()();
                if !comparator(&memo.get_untracked(), &new_value) {
                    memo.set(new_value);
                }
            }
        };

        (Box::new(effect), memo.into_handle())
    })
}

/// Run the passed closure inside an untracked scope.
///
/// See also [`StateHandle::get_untracked()`].
///
/// # Example
///
/// ```
/// use sycamore::prelude::*;
///
/// let state = Signal::new(1);
///
/// let double = create_memo({
///     let state = state.clone();
///     move || untrack(|| *state.get() * 2)
/// });
///
/// assert_eq!(*double.get(), 2);
///
/// state.set(2);
/// // double value should still be old value because state was untracked
/// assert_eq!(*double.get(), 2);
/// ```
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    let f = Rc::new(RefCell::new(Some(f)));
    let g = Rc::clone(&f);

    // Do not panic if running inside destructor.
    if let Ok(ret) = CONTEXTS.try_with(|contexts| {
        let tmp = contexts.take();

        let ret = f.take().unwrap()();

        *contexts.borrow_mut() = tmp;

        ret
    }) {
        ret
    } else {
        g.take().unwrap()()
    }
}

/// Adds a callback function to the current reactive scope's cleanup.
///
/// # Example
/// ```
/// use sycamore::prelude::*;
///
/// let cleanup_called = Signal::new(false);
///
/// let scope = create_root(cloned!((cleanup_called) => move || {
///     on_cleanup(move || {
///         cleanup_called.set(true);
///     })
/// }));
///
/// assert_eq!(*cleanup_called.get(), false);
///
/// drop(scope);
/// assert_eq!(*cleanup_called.get(), true);
/// ```
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    SCOPE.with(|scope| {
        if scope.borrow().is_some() {
            scope
                .borrow_mut()
                .as_mut()
                .unwrap()
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

/// Gets the number of dependencies of the current reactive scope.
///
/// If the function is called outside a reactive scope, it will return `None`.
///
/// # Example
/// ```
/// use sycamore::prelude::*;
/// use sycamore::rx::dependency_count;
///
/// assert_eq!(dependency_count(), None);
///
/// let state = Signal::new(1);
/// create_effect(move || {
///     assert_eq!(dependency_count(), Some(0));
///     state.get();
///     assert_eq!(dependency_count(), Some(1));
/// });
/// ```
pub fn dependency_count() -> Option<usize> {
    CONTEXTS.with(|contexts| {
        contexts.borrow().last().map(|last_context| {
            last_context
                .upgrade()
                .expect("Running should be valid while inside reactive scope")
                .borrow()
                .as_ref()
                .unwrap()
                .dependencies
                .len()
        })
    })
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
    fn nested_effects_trigger_outer_effect() {
        let trigger = Signal::new(());

        let outer_counter = Signal::new(0);
        let inner_counter = Signal::new(0);
        let inner_cleanup_counter = Signal::new(0);

        create_effect(
            cloned!((trigger, outer_counter, inner_counter, inner_cleanup_counter) => move || {
                trigger.get(); // subscribe to trigger
                outer_counter.set(*outer_counter.get_untracked() + 1);

                create_effect(cloned!((trigger, inner_counter, inner_cleanup_counter) => move || {
                    trigger.set(()); // update trigger which should recreate the outer effect
                    inner_counter.set(*inner_counter.get_untracked() + 1);

                    on_cleanup(cloned!((inner_cleanup_counter) => move || {
                        inner_cleanup_counter.set(*inner_cleanup_counter.get_untracked() + 1);
                    }));
                }));
            }),
        );

        assert_eq!(*outer_counter.get(), 1);
        assert_eq!(*inner_counter.get(), 1);
        assert_eq!(*inner_cleanup_counter.get(), 0);

        trigger.set(());

        assert_eq!(*outer_counter.get(), 2);
        assert_eq!(*inner_counter.get(), 2);
        assert_eq!(*inner_cleanup_counter.get(), 1);
    }

    #[test]
    fn create_nested_effect_from_outside() {
        let trigger = Signal::new(());
        let outer_counter = Signal::new(0);
        let inner_counter = Signal::new(0);

        let inner_effect: Signal<Option<Box<dyn Fn()>>> = Signal::new(None);

        create_effect(
            cloned!((trigger, outer_counter, inner_counter, inner_effect) => move || {
                trigger.get(); // subscribe to trigger
                outer_counter.set(*outer_counter.get_untracked() + 1);

                if inner_effect.get_untracked().is_none() {
                    inner_effect.set(Some(Box::new(cloned!((inner_counter) => move || {
                        inner_counter.set(*inner_counter.get_untracked() + 1);
                    }))));
                }
            }),
        );

        create_effect(move || (*inner_effect.get()).as_ref().unwrap()());

        assert_eq!(*outer_counter.get(), 1);
        assert_eq!(*inner_counter.get(), 1);

        trigger.set(());
        assert_eq!(*outer_counter.get(), 2);
        assert_eq!(*inner_counter.get(), 1);
    }

    #[test]
    fn outer_effects_rerun_first() {
        let trigger = Signal::new(());

        let outer_counter = Signal::new(0);
        let inner_counter = Signal::new(0);

        create_effect(cloned!((trigger, outer_counter, inner_counter) => move || {
            trigger.get(); // subscribe to trigger
            outer_counter.set(*outer_counter.get_untracked() + 1);

            create_effect(cloned!((trigger, inner_counter) => move || {
                trigger.get(); // subscribe to trigger
                inner_counter.set(*inner_counter.get_untracked() + 1);
            }));
        }));

        assert_eq!(*outer_counter.get(), 1);
        assert_eq!(*inner_counter.get(), 1);

        trigger.set(());

        assert_eq!(*outer_counter.get(), 2);
        assert_eq!(*inner_counter.get(), 2);
    }

    #[test]
    fn drop_signal_inside_effect() {
        let state = RefCell::new(Some(Signal::new(0)));

        create_effect(move || {
            if let Some(state) = state.take() {
                state.get(); // subscribe to state
                drop(state);
            }
        });
    }

    #[test]
    fn destroy_effects_on_scope_drop() {
        let counter = Signal::new(0);

        let trigger = Signal::new(());

        let scope = create_root(cloned!((trigger, counter) => move || {
            create_effect(move || {
                trigger.get(); // subscribe to trigger
                counter.set(*counter.get_untracked() + 1);
            });
        }));

        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 2);

        drop(scope);
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
        assert_eq!(*double.get(), 2); // double value should still be true because state.get() was
                                      // inside untracked
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

    #[test]
    fn cleanup() {
        let cleanup_called = Signal::new(false);
        let scope = create_root(cloned!((cleanup_called) => move || {
            on_cleanup(move || {
                cleanup_called.set(true);
            });
        }));

        assert!(!*cleanup_called.get());

        drop(scope);
        assert!(*cleanup_called.get());
    }

    #[test]
    fn cleanup_in_effect() {
        let trigger = Signal::new(());

        let counter = Signal::new(0);

        create_effect(cloned!((trigger, counter) => move || {
            trigger.get(); // subscribe to trigger

            on_cleanup(cloned!((counter) => move || {
                counter.set(*counter.get() + 1);
            }));
        }));

        assert_eq!(*counter.get(), 0);

        trigger.set(());
        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 2);
    }

    #[test]
    fn cleanup_is_untracked() {
        let trigger = Signal::new(());

        let counter = Signal::new(0);

        create_effect(cloned!((trigger, counter) => move || {
            counter.set(*counter.get_untracked() + 1);

            on_cleanup(cloned!((trigger) => move || {
                trigger.get(); // do not subscribe to trigger
            }));
        }));

        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 1);
    }
}

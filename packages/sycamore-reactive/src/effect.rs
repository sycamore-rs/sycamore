//! Side effects.

use std::collections::HashSet;

use crate::*;

thread_local! {
    /// While the [`EffectState`] is inside the Vec, it is owned by [`EFFECTS`].
    /// Because this is a global variable, the lifetime is necessarily `'static`. However, that does not mean
    /// that it can last forever. The `EffectState` should only be used the time it is inside [`EFFECTS`].
    pub(crate) static EFFECTS: RefCell<Vec<*mut EffectState<'static>>> = Default::default();
}

/// The internal state of an effect. The effect callback and the effect dependencies are stored in
/// this struct.
pub(crate) struct EffectState<'a> {
    /// The callback when the effect is re-executed.
    cb: Rc<RefCell<dyn FnMut() + 'a>>,
    /// A list of dependencies that can trigger this effect.
    dependencies: HashSet<EffectDependency<'a>>,
}

/// Implements reference equality for [`AnySignal`]s.
pub(crate) struct EffectDependency<'a>(&'a SignalEmitter);
impl<'a> std::cmp::PartialEq for EffectDependency<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl<'a> std::cmp::Eq for EffectDependency<'a> {}
impl<'a> std::hash::Hash for EffectDependency<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const SignalEmitter).hash(state);
    }
}

impl<'a> EffectState<'a> {
    // Clears the dependencies (both links and backlinks).
    /// Should be called when re-executing an effect to recreate all dependencies.
    pub fn clear_dependencies(&mut self) {
        for dependency in &self.dependencies {
            // SAFETY: TODO
            dependency
                .0
                .unsubscribe(unsafe { std::mem::transmute(Rc::as_ptr(&self.cb)) });
        }
        self.dependencies.clear();
    }

    /// Add a dependency to the effect.
    pub fn add_dependency(&mut self, signal: &'a SignalEmitter) {
        self.dependencies.insert(EffectDependency(signal));
    }
}

impl<'a> Scope<'a> {
    /// Creates an effect on signals used inside the effect closure.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(0);
    ///
    /// ctx.create_effect(|| {
    ///     println!("State changed. New state value = {}", state.get());
    /// }); // Prints "State changed. New state value = 0"
    ///
    /// state.set(1); // Prints "State changed. New state value = 1"
    /// # });
    /// ```
    pub fn create_effect(&self, f: impl FnMut() + 'a) {
        self._create_effect(Box::new(f))
    }

    /// Internal implementation for `create_effect`. Use dynamic dispatch to reduce code-bloat.
    fn _create_effect(&self, mut f: Box<dyn FnMut() + 'a>) {
        let effect = Rc::new(RefCell::new(None::<EffectState<'a>>));
        let cb = Rc::new(RefCell::new({
            let effect = Rc::downgrade(&effect);
            move || {
                EFFECTS.with(|effects| {
                    // Record initial effect stack length to verify that it is the same after.
                    let initial_effect_stack_len = effects.borrow().len();
                    // Upgrade the effect to an Rc now so that it is valid for the rest of the
                    // callback.
                    let effect_ref = effect.upgrade().unwrap();

                    // Take effect out.
                    let mut effect = effect_ref.take().unwrap();
                    effect.clear_dependencies();

                    // Push the effect onto the effect stack.
                    let boxed = Box::new(effect);
                    let ptr: *mut EffectState<'a> = Box::into_raw(boxed);
                    // Push the effect onto the effect stack so that it is visible by signals.
                    effects
                        .borrow_mut()
                        .push(ptr as *mut () as *mut EffectState<'static>);
                    // Now we can call the user-provided function.
                    f();
                    // Pop the effect from the effect stack.
                    effects.borrow_mut().pop().unwrap();

                    //  SAFETY: Now that the effect has been popped from EFFECTS,
                    // get a boxed EffectState with the correct lifetime back.
                    let boxed = unsafe { Box::from_raw(ptr) };

                    // For all the signals collected by the EffectState,
                    // we need to add backlinks from the signal to the effect, so that
                    // updating the signal will trigger the effect.
                    for emitter in &boxed.dependencies {
                        // SAFETY: When the effect is destroyed or when the emitter is dropped, this
                        // link will be destroyed to prevent dangling references.
                        emitter
                            .0
                            .subscribe(unsafe { std::mem::transmute(Rc::downgrade(&boxed.cb)) });
                    }

                    // Get the effect state back into the Rc
                    *effect_ref.borrow_mut() = Some(*boxed);

                    debug_assert_eq!(effects.borrow().len(), initial_effect_stack_len);
                });
            }
        }));

        // Initialize initial effect state.
        *effect.borrow_mut() = Some(EffectState {
            cb: cb.clone(),
            dependencies: HashSet::new(),
        });

        // Initial callback call to get everything started.
        cb.borrow_mut()();

        // Push Rc to self.effects so that it is not dropped immediately.
        self.effects.borrow_mut().push(effect);
    }

    /// Creates an effect on signals used inside the effect closure.
    ///
    /// Instead of [`create_effect`](Self::create_effect), this function also provides a new
    /// reactive scope instead the effect closure. This scope is created for each new run of the
    /// effect.
    ///
    /// Items created within the scope cannot escape outside the effect because that can result in
    /// an use-after-free.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// ctx.create_effect_scoped(|ctx| {
    ///     // Use the scoped ctx inside here.
    ///     let _nested_signal = ctx.create_signal(0);
    ///     // _nested_signal cannot escape out of the effect closure.
    /// });
    /// # });
    /// ```
    pub fn create_effect_scoped<F>(&'a self, mut f: F)
    where
        F: for<'child_lifetime> FnMut(BoundedScopeRef<'child_lifetime, 'a>) + 'a,
    {
        let mut disposer: Option<Box<dyn FnOnce()>> = None;
        self.create_effect(move || {
            // We run the disposer inside the effect, after effect dependencies have been cleared.
            // This is to make sure that if the effect subscribes to its own signal, there is no
            // use-after-free during the clear dependencies phase.
            if let Some(disposer) = disposer.take() {
                disposer();
            }
            // Create a new nested scope and save the disposer.
            // SAFETY: The whole point. We want the scope to be tracked so that signals accessed
            // from within are picked-up by the effect. This is safe because `disposer` is only
            // called at the beginning of this `create_effect` call after clear dependencies phase
            // and not before closure returns with new scope.
            let new_disposer: Option<Box<dyn FnOnce()>> = Some(Box::new(unsafe {
                self.create_child_scope_tracked(|ctx| {
                    // SAFETY: f takes the same parameter as the argument to
                    // self.create_child_scope(_).
                    f(std::mem::transmute(ctx))
                })
            }));
            // SAFETY: transmute the lifetime. This is safe because disposer is only used within the
            // effect which is necessarily within the lifetime of self (the Scope).
            disposer = unsafe { std::mem::transmute(new_disposer) };
        });
    }
}

/// Run the passed closure inside an untracked dependency scope.
///
/// See also [`ReadSignal::get_untracked()`].
///
/// # Example
///
/// ```
/// # use sycamore_reactive::*;
/// # create_scope_immediate(|ctx| {
/// let state = ctx.create_signal(1);
/// let double = ctx.create_memo(|| untrack(|| *state.get() * 2));
/// //                              ^^^^^^^
/// assert_eq!(*double.get(), 2);
///
/// state.set(2);
/// // double value should still be old value because state was untracked
/// assert_eq!(*double.get(), 2);
/// # });
/// ```
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    EFFECTS.with(|effects| {
        let tmp = effects.take();
        let ret = f();
        *effects.borrow_mut() = tmp;
        ret
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);

            let double = ctx.create_signal(-1);

            ctx.create_effect(|| {
                double.set(*state.get() * 2);
            });
            assert_eq!(*double.get(), 0); // calling create_effect should call the effect at least once

            state.set(1);
            assert_eq!(*double.get(), 2);
            state.set(2);
            assert_eq!(*double.get(), 4);
        });
    }

    #[test]
    fn effect_with_explicit_dependencies() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);

            let double = ctx.create_signal(-1);

            ctx.create_effect(on([state], || {
                double.set(*state.get() * 2);
            }));
            assert_eq!(*double.get(), 0); // calling create_effect should call the effect at least once

            state.set(1);
            assert_eq!(*double.get(), 2);
            state.set(2);
            assert_eq!(*double.get(), 4);
        });
    }

    #[test]
    fn effect_cannot_create_infinite_loop() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            ctx.create_effect(|| {
                state.track();
                state.set(0);
            });
            state.set(0);
        });
    }

    #[test]
    fn effect_should_only_subscribe_once_to_same_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);

            let counter = ctx.create_signal(0);
            ctx.create_effect(|| {
                counter.set(*counter.get_untracked() + 1);

                // call state.track() twice but should subscribe once
                state.track();
                state.track();
            });

            assert_eq!(*counter.get(), 1);

            state.set(1);
            assert_eq!(*counter.get(), 2);
        });
    }

    #[test]
    fn effect_should_recreate_dependencies_each_time() {
        create_scope_immediate(|ctx| {
            let condition = ctx.create_signal(true);

            let state1 = ctx.create_signal(0);
            let state2 = ctx.create_signal(1);

            let counter = ctx.create_signal(0);
            ctx.create_effect(|| {
                counter.set(*counter.get_untracked() + 1);

                if *condition.get() {
                    state1.track();
                } else {
                    state2.track();
                }
            });

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
        });
    }

    #[test]
    fn outer_effects_run_first() {
        create_scope_immediate(|ctx| {
            let trigger = ctx.create_signal(());

            let outer_counter = ctx.create_signal(0);
            let inner_counter = ctx.create_signal(0);

            ctx.create_effect_scoped(|ctx| {
                trigger.track();
                outer_counter.set(*outer_counter.get_untracked() + 1);

                ctx.create_effect(|| {
                    trigger.track();
                    inner_counter.set(*inner_counter.get_untracked() + 1);
                });
            });

            assert_eq!(*outer_counter.get(), 1);
            assert_eq!(*inner_counter.get(), 1);

            trigger.set(());

            assert_eq!(*outer_counter.get(), 2);
            assert_eq!(*inner_counter.get(), 2);
        });
    }

    #[test]
    fn destroy_effects_on_scope_dispose() {
        create_scope_immediate(|ctx| {
            let counter = ctx.create_signal(0);

            let trigger = ctx.create_signal(());

            let disposer = ctx.create_child_scope(|ctx| {
                ctx.create_effect(|| {
                    trigger.track();
                    counter.set(*counter.get_untracked() + 1);
                });
            });

            assert_eq!(*counter.get(), 1);

            trigger.set(());
            assert_eq!(*counter.get(), 2);

            disposer();
            trigger.set(());
            assert_eq!(*counter.get(), 2); // inner effect should be destroyed and thus not executed
        });
    }

    #[test]
    fn effect_preserves_scope_hierarchy() {
        create_scope_immediate(|ctx| {
            let trigger = ctx.create_signal(());
            let parent: &Signal<Option<*const ()>> = ctx.create_signal(None);
            ctx.create_effect_scoped(|ctx| {
                trigger.track();
                let p = ctx.parent.unwrap();
                parent.set(Some(p as *const ()));
            });
            assert_eq!(
                parent.get().unwrap(),
                ctx as *const _ as *const (),
                "the parent scope of the effect should be `ctx`"
            );
            trigger.set(());
            assert_eq!(
                parent.get().unwrap(),
                ctx as *const _ as *const (),
                "the parent should still be `ctx` after effect is re-executed"
            );
        });
    }

    #[test]
    fn effect_scoped_subscribing_to_own_signal() {
        create_scope_immediate(|ctx| {
            let trigger = ctx.create_signal(());
            ctx.create_effect_scoped(|ctx| {
                trigger.track();
                let signal = ctx.create_signal(());
                // Track own signal:
                signal.track();
            });
            trigger.set(());
        });
    }
}

//! Side effects!

use std::cell::RefCell;
use std::rc::Rc;

use crate::{create_child_scope, create_memo, Root};

/// Creates an effect on signals used inside the effect closure.
///
/// # Example
/// ```
/// # use sycamore_reactive::*;
/// # create_root(|| {
/// let state = create_signal(0);
///
/// create_effect(move || {
///     println!("State changed. New state value = {}", state.get());
/// });
/// // Prints "State changed. New state value = 0"
///
/// state.set(1);
/// // Prints "State changed. New state value = 1"
/// # });
/// ```
///
/// `create_effect` should only be used for creating **side-effects**. It is generally not
/// recommended to update signal states inside an effect. You probably should be using a
/// [`create_memo`](crate::create_memo) instead.
pub fn create_effect(f: impl FnMut() + 'static) {
    let f = Rc::new(RefCell::new(f));
    create_memo(move || {
        let f = Rc::clone(&f);
        Root::global().call_effect(move || f.borrow_mut()());
    });
}

/// Creates an effect on signals used inside the effect closure.
///
/// Unlike [`create_effect`], this function also provides a new reactive scope instead the
/// effect closure. This scope is created for each new run of the effect.
///
/// # Example
/// ```
/// # use sycamore_reactive::*;
/// # create_root(|| {
/// create_effect_scoped(|| {
///     // Use the scoped cx inside here.
///     let _nested_signal = create_signal(0);
///     // _nested_signal cannot escape out of the effect closure.
/// });
/// # });
/// ```
#[deprecated(note = "same as create_effect")]
pub fn create_effect_scoped(f: impl FnMut() + 'static) {
    create_effect(f);
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn effect() {
        let _ = create_root(|| {
            let state = create_signal(0);

            let double = create_signal(-1);

            create_effect(move || {
                double.set(state.get() * 2);
            });
            assert_eq!(double.get(), 0); // calling create_effect should call the effect at least once

            state.set(1);
            assert_eq!(double.get(), 2);
            state.set(2);
            assert_eq!(double.get(), 4);
        });
    }

    #[test]
    fn effect_with_explicit_dependencies() {
        let _ = create_root(|| {
            let state = create_signal(0);

            let double = create_signal(-1);

            create_effect(on(state, move || {
                double.set(state.get() * 2);
            }));
            assert_eq!(double.get(), 0); // calling create_effect should call the effect at least once

            state.set(1);
            assert_eq!(double.get(), 2);
            state.set(2);
            assert_eq!(double.get(), 4);
        });
    }

    #[test]
    fn effect_cannot_create_infinite_loop() {
        let _ = create_root(|| {
            let state = create_signal(0);
            create_effect(move || {
                state.track();
                state.set(0);
            });
            state.set(0);
        });
    }

    #[test]
    fn effect_should_only_subscribe_once_to_same_signal() {
        let _ = create_root(|| {
            let state = create_signal(0);

            let counter = create_signal(0);
            create_effect(move || {
                counter.set(counter.get_untracked() + 1);

                // call state.track() twice but should subscribe once
                state.track();
                state.track();
            });

            assert_eq!(counter.get(), 1);

            state.set(1);
            assert_eq!(counter.get(), 2);
        });
    }

    #[test]
    fn effect_should_recreate_dependencies_each_time() {
        let _ = create_root(|| {
            let condition = create_signal(true);

            let state1 = create_signal(0);
            let state2 = create_signal(1);

            let counter = create_signal(0);
            create_effect(move || {
                counter.set(counter.get_untracked() + 1);

                if condition.get() {
                    state1.track();
                } else {
                    state2.track();
                }
            });

            assert_eq!(counter.get(), 1);

            state1.set(1);
            assert_eq!(counter.get(), 2);

            state2.set(1);
            assert_eq!(counter.get(), 2); // not tracked

            condition.set(false);
            assert_eq!(counter.get(), 3);

            state1.set(2);
            assert_eq!(counter.get(), 3); // not tracked

            state2.set(2);
            assert_eq!(counter.get(), 4); // tracked after condition.set
        });
    }

    #[test]
    fn inner_effects_run_first() {
        let _ = create_root(|| {
            let trigger = create_signal(());

            let outer_counter = create_signal(0);
            let inner_counter = create_signal(0);

            create_effect_scoped(move || {
                trigger.track();
                outer_counter.set(outer_counter.get_untracked() + 1);

                create_effect(move || {
                    trigger.track();
                    inner_counter.set(inner_counter.get_untracked() + 1);
                });
            });

            assert_eq!(outer_counter.get(), 1);
            assert_eq!(inner_counter.get(), 1);

            trigger.set(());

            assert_eq!(outer_counter.get(), 2);
            assert_eq!(inner_counter.get(), 3);
        });
    }

    #[test]
    fn destroy_effects_on_scope_dispose() {
        let _ = create_root(|| {
            let counter = create_signal(0);

            let trigger = create_signal(());

            let child_scope = create_child_scope(move || {
                create_effect(move || {
                    trigger.track();
                    counter.set(counter.get_untracked() + 1);
                });
            });

            assert_eq!(counter.get(), 1);

            trigger.set(());
            assert_eq!(counter.get(), 2);

            child_scope.dispose();
            trigger.set(());
            assert_eq!(counter.get(), 2); // inner effect should be destroyed and thus not executed
        });
    }

    #[test]
    fn effect_scoped_subscribing_to_own_signal() {
        let _ = create_root(|| {
            let trigger = create_signal(());
            create_effect_scoped(move || {
                trigger.track();
                let signal = create_signal(());
                // Track own signal:
                signal.track();
            });
            trigger.set(());
        });
    }
}

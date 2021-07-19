//! Side effects and derived signals.

use std::cell::RefCell;
use std::collections::HashSet;
use std::mem;
use std::rc::Rc;

use crate::scope::{create_root, ReactiveScope};
use crate::signal::SignalId;

thread_local! {
    /// The current effect listener or `None`.
    pub(crate) static CURRENT_LISTENER: RefCell<Option<Listener>> = RefCell::new(None);
}

pub(crate) struct EffectState {
    callback: Rc<RefCell<dyn FnMut()>>,
    dependencies: HashSet<SignalId>,
    scope: Option<ReactiveScope>,
}

impl EffectState {
    /// Rerun the effect.
    pub fn trigger(&self) {
        self.callback.borrow_mut()();
    }
}

pub(crate) struct Listener(Rc<RefCell<Option<EffectState>>>);

impl Listener {
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

pub fn create_effect(mut f: impl FnMut() + 'static) {
    let effect_state = Rc::new(RefCell::new(None));

    // Callback for when the effect's dependencies are triggered.
    let callback: Rc<RefCell<dyn FnMut()>> = Rc::new(RefCell::new({
        let effect_state = Rc::clone(&effect_state);
        move || {
            CURRENT_LISTENER.with(|listener| {
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
    debug_assert_eq!(Rc::strong_count(&effect_state), 1);

    // Effect always calls the callback once.
    callback.borrow_mut()();
}

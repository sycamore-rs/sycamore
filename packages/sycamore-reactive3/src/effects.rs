//! Side effects!

use slotmap::new_key_type;

use crate::Scope;

new_key_type! { pub(crate) struct EffectId; }

pub(crate) struct EffectState {
    /// The callback of the effect. This is an `Option` so that we can temporarily take the
    /// callback out to call it without holding onto a mutable borrow of all the effects.
    pub callback: Option<Box<dyn FnMut()>>,
    /// An internal state to prevent an effect from running twice in the same update.
    pub already_run_in_update: bool,
}

/// Creates an effect on signals used inside the effect closure.
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 0);
///
/// create_effect(cx, move || {
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
/// [`create_memo`] instead.
pub fn create_effect(cx: Scope, mut f: impl FnMut() + 'static) {
    // Run the effect right now so we can get the dependencies.
    let (_, tracker) = cx.root.tracked_scope(&mut f);
    let data = EffectState {
        callback: Some(Box::new(f)),
        already_run_in_update: false,
    };
    let key = cx.root.effects.borrow_mut().insert(data);
    // Add the dependency links.
    tracker.create_effect_dependency_links(cx.root, key);
}

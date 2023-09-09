//! Reactive primitives for [Sycamore](https://github.com/sycamore-rs/sycamore).
//!
//! ```rust
//! use sycamore_reactive3::*;
//!
//! create_root(|cx| {
//!     let greeting = create_signal(cx, "Hello");
//!     let name = create_signal(cx, "World");
//!
//!     let display_text = create_memo(cx, move || format!("{greeting} {name}!"));
//!     assert_eq!(display_text.get_clone(), "Hello World!");
//!
//!     name.set("Sycamore");
//!     assert_eq!(display_text.get_clone(), "Hello Sycamore!");
//! });
//! ```
//!
//! # A note on `nightly`
//!
//! If you are using rust `nightly`, you can enable the `nightly` feature to enable the more terse
//! syntax for signal get/set.
//!
//! ```rust
//! # use sycamore_reactive3::*;
//! # create_root(|cx| {
//! let signal = create_signal(cx, 123);
//!
//! // Stable:
//! let value = signal.get();
//! signal.set(456);
//!
//! // Nightly:
//! let value = signal();
//! signal(456);
//! # });
//! ```
//! Of course, the stable `.get()` also works on nightly as well if that's what you prefer.

#![cfg_attr(feature = "nightly", feature(fn_traits))]
#![cfg_attr(feature = "nightly", feature(unboxed_closures))]

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::sync::Mutex;

use signals::{Mark, SignalId, SignalState};
use slotmap::{new_key_type, SlotMap};

mod context;
mod effects;
mod iter;
mod memos;
mod signals;
mod store;
mod utils;

pub use context::*;
pub use effects::*;
pub use iter::*;
pub use memos::*;
pub use signals::*;
pub use store::*;
pub use utils::*;

/// Add name for proc-macro purposes.
extern crate self as sycamore_reactive3;

/// The struct managing the state of the reactive system. Only one should be created per running
/// app.
///
/// Often times, this is intended to be leaked to be able to get a `&'static Root`. However, the
/// `Root` is also `dispose`-able, meaning that any resources allocated in this `Root` will get
/// deallocated. Therefore in practice, there should be no memory leak at all except for the `Root`
/// itself. Finally, the `Root` is expected to live for the whole duration of the app so this is
/// not a problem.
#[derive(Default)]
struct Root {
    /// A reference to the root scope.
    root_scope: Cell<ScopeId>,
    /// All the scopes that have been created in this `Root`.
    scopes: RefCell<SlotMap<ScopeId, ScopeState>>,
    /// All the signals that have been created in this `Root`.
    /// Eventhough signals are stored here, they are created under roots and are destroyed by the
    /// scopes when they themselves are dropped.
    signals: RefCell<SlotMap<SignalId, SignalState>>,
    /// ALl the effects that have been created in this `Root`.
    /// Effects are also destroyed by the scopes when they themselves are dropped.
    effects: RefCell<SlotMap<EffectId, EffectState>>,
    /// If this is `Some`, that means we are tracking signal accesses.
    tracker: RefCell<Option<DependencyTracker>>,
    /// A temporary buffer used in `propagate_updates` to prevent allocating a new Vec every time
    /// it is called.
    rev_sorted_buf: RefCell<Vec<SignalId>>,
    /// A list of effects to be run after signal updates are over.
    effect_queue: RefCell<Vec<EffectId>>,
    /// Whether we are currently batching signal updatse. If this is true, we do not run
    /// `effect_queue` and instead wait until the end of the batch.
    batch: Cell<bool>,
}

impl Root {
    /// Run the provided closure in a tracked scope. This will detect all the signals that are
    /// accessed and track them in a dependency list.
    fn tracked_scope<T>(&self, f: impl FnOnce() -> T) -> (T, DependencyTracker) {
        let prev = self.tracker.replace(Some(DependencyTracker::default()));
        let ret = f();
        (ret, self.tracker.replace(prev).unwrap())
    }

    /// Run the update callback of the signal, also recreating any dependencies found by
    /// tracking signal accesses inside the function. This method does _not_ delete existing
    /// dependency links.
    ///
    /// # Params
    /// * `root` - The reactive root.
    /// * `id` - The ID associated with this `SignalState`. This is because we are not storing the
    /// `SignalId` inside the state itself.
    ///
    /// # Returns
    /// Returns whether the signal value has been changed.
    fn run_signal_update(&self, id: SignalId) -> bool {
        let dependencies = std::mem::take(&mut self.signals.borrow_mut()[id].dependencies);
        for dependency in dependencies {
            self.signals.borrow_mut()[dependency]
                .dependents
                .retain(|&x| x != id);
        }
        // We take the update callback out because that requires a mut ref and we cannot hold that
        // while running update itself.
        let mut update = self.signals.borrow_mut()[id].update.take();
        let changed = if let Some(update) = &mut update {
            let mut value = self.signals.borrow()[id].value.take().unwrap();
            let (changed, tracker) = self.tracked_scope(|| update(&mut value));
            *self.signals.borrow()[id].value.borrow_mut() = Some(value);
            tracker.create_signal_dependency_links(self, id);
            changed
        } else {
            false
        };
        // Put the update back in.
        self.signals.borrow_mut()[id].update = update;
        changed
    }

    /// Run the callback of the effect, also recreating any dependencies found by tracking signal
    /// accesses inside the function. This method does _not_ delete existing dependency links.
    ///
    /// # Params
    /// * `root` - The reactive root.
    /// * `id` - The ID associated with this `EffectState`. This is because we are not storing the
    /// `EffectId` inside the state itself.
    fn run_effect_update(&self, id: EffectId) {
        for dependency in self.effects.borrow_mut()[id].dependencies.drain(..) {
            self.signals.borrow_mut()[dependency]
                .effect_dependents
                .retain(|&x| x != id);
        }
        // We take the update callback out because that requires a mut ref and we cannot hold that
        // while running the callback itself.
        let mut callback = self.effects.borrow_mut()[id]
            .callback
            .take()
            .expect("callback should not be None");
        let (_, tracker) = self.tracked_scope(&mut callback);
        tracker.create_effect_dependency_links(self, id);
        // Put the callback back in.
        self.effects.borrow_mut()[id].callback = Some(callback);
    }

    /// Runs and clears all the effects in `effect_queue`.
    fn run_effects(&self) {
        // 1 - Reset all values for `already_run_in_update`
        let effect_queue = self.effect_queue.take();
        for &effect_id in &effect_queue {
            // Filter out all the effects that are already dead.
            if let Some(effect) = self.effects.borrow_mut().get_mut(effect_id) {
                effect.already_run_in_update = false;
            }
        }
        // 2 - Run all the effects.
        for &effect_id in &effect_queue {
            let mut effects_mut = self.effects.borrow_mut();
            // Filter out all the effects that are already dead.
            if let Some(effect) = effects_mut.get_mut(effect_id) {
                if !effect.already_run_in_update {
                    // Prevent effects from running twice.
                    effect.already_run_in_update = true;
                    drop(effects_mut); // We can't hold on to self.effects because a signal might
                                       // be set inside the effect.
                    self.run_effect_update(effect_id);
                }
            }
        }
    }

    /// If there are no cyclic dependencies, then the reactive graph is a DAG (Directed Acylic
    /// Graph). We can therefore use DFS to get a topological sorting of all the reactive nodes.
    ///
    /// We then go through every node in this topological sorting and update only those nodes which
    /// have dependencies that were updated. TODO: Is there a way to cut update short if nothing
    /// changed?
    #[cfg_attr(debug_assertions, track_caller)]
    fn propagate_signal_updates(&self, start_node: SignalId) {
        // Avoid allocation by reusing a `Vec` stored in the `Root`.
        let mut rev_sorted = self
            .rev_sorted_buf
            .try_borrow_mut()
            .expect("cannot update a signal inside a memo");
        rev_sorted.clear();

        self.dfs(start_node, &mut self.signals.borrow_mut(), &mut rev_sorted);

        for &node in rev_sorted.iter().rev() {
            let mut signals_mut = self.signals.borrow_mut();
            let node_state = &mut signals_mut[node];
            node_state.mark = Mark::None; // Reset value.

            // Do not update the starting node since it has already been updated.
            if node == start_node {
                node_state.changed_in_last_update = true;
                self.effect_queue
                    .borrow_mut()
                    .extend(node_state.effect_dependents.drain(..));
                continue;
            }
            drop(signals_mut);

            // Check if dependencies are updated.
            let any_dep_changed = self.signals.borrow()[node]
                .dependencies
                .iter()
                .any(|dep| self.signals.borrow()[*dep].changed_in_last_update);

            let changed = if any_dep_changed {
                // Both dependencies and dependents have been erased by now.
                self.run_signal_update(node)
            } else {
                false
            };
            let mut signals_mut = self.signals.borrow_mut();
            let node_state = &mut signals_mut[node];
            node_state.changed_in_last_update = changed;

            // If the signal value has changed, add all the effects that depend on it to the effect
            // queue.
            if changed {
                self.effect_queue
                    .borrow_mut()
                    .extend(node_state.effect_dependents.drain(..));
            }
        }
    }

    /// Call this if `start_node` has been updated manually. This will automatically update all
    /// signals that depend on `start_node` as well as call any effects as necessary.
    #[cfg_attr(debug_assertions, track_caller)]
    fn propagate_updates(&self, start_node: SignalId) {
        // Propagate any signal updates.
        self.propagate_signal_updates(start_node);
        if !self.batch.get() {
            // Run all the effects that have been queued.
            self.run_effects();
        }
    }

    /// Run depth-first-search on the reactive graph starting at `current`.
    ///
    /// Also resets `changed_in_last_update` and adds a [`Mark::Permanent`] for all signals
    /// traversed.
    fn dfs(
        &self,
        current_id: SignalId,
        signals: &mut SlotMap<SignalId, SignalState>,
        buf: &mut Vec<SignalId>,
    ) {
        let Some(current) = signals.get_mut(current_id) else {
            // If signal is dead, don't even visit it.
            return;
        };

        current.changed_in_last_update = false; // Reset value.
        match current.mark {
            Mark::Temp => panic!("cylcic reactive dependency detected"),
            Mark::Permanent => return,
            Mark::None => {}
        }
        current.mark = Mark::Temp;

        let children = current.dependents.clone();
        current.dependents.clear();
        for child in children {
            self.dfs(child, signals, buf);
        }
        signals[current_id].mark = Mark::Permanent;
        buf.push(current_id);
    }

    /// Sets the batch flag to `true`.
    fn start_batch(&self) {
        self.batch.set(true);
    }

    /// Sets the batch flag to `false` and run all the queued effects.
    fn end_batch(&self) {
        self.batch.set(false);
        self.run_effects();
    }
}

/// A handle to a root. This lets you reinitialize or dispose the root for resource cleanup.
///
/// This is generally obtained from [`create_root`].
#[derive(Clone, Copy)]
pub struct RootHandle {
    _ref: &'static Root,
}

impl RootHandle {
    /// Reinitializes the root. Once the root is reinitialized, nothing from before this call
    /// should reference this `Root`.
    pub fn reinitialize(&self, mut f: impl FnMut(Scope)) {
        // Destroy everything!
        let _ = self._ref.scopes.take();
        let _ = self._ref.signals.take();
        let _ = self._ref.effects.take();
        let _ = self._ref.tracker.take();
        let _ = self._ref.rev_sorted_buf.take();
        let _ = self._ref.effect_queue.take();

        // Create an initial scope and call our callback.
        let root_scope = ScopeState::new(self._ref, None);
        let root_scope_key = self._ref.scopes.borrow_mut().insert(root_scope);
        self._ref.root_scope.set(root_scope_key);

        let cx = Scope {
            id: root_scope_key,
            root: self._ref,
        };
        f(cx);
    }

    /// Destroy everything that was created in this scope. This is simply an alias for
    /// [`RootHandle::reinitialize`] with an empty callback.
    pub fn dispose(&self) {
        self.reinitialize(|_| {})
    }
}

/// Tracks signals that are accessed inside a reactive scope.
#[derive(Default)]
struct DependencyTracker {
    /// A list of signals that were accessed.
    dependencies: Vec<SignalId>,
}

impl DependencyTracker {
    /// Sets the `dependents` field for all the signals that have been tracked.
    fn create_signal_dependency_links(self, root: &Root, dependent: SignalId) {
        for signal in &self.dependencies {
            root.signals.borrow_mut()[*signal]
                .dependents
                .push(dependent);
        }
        // Set the signal dependencies so that it is updated automatically.
        root.signals.borrow_mut()[dependent].dependencies = self.dependencies;
    }

    /// Sets the `effect_dependents` field for all the signals that have been tracked.
    fn create_effect_dependency_links(self, root: &Root, dependent: EffectId) {
        for signal in &self.dependencies {
            root.signals.borrow_mut()[*signal]
                .effect_dependents
                .push(dependent);
        }
        root.effects.borrow_mut()[dependent].dependencies = self.dependencies;
    }
}

new_key_type! {
    /// Id for [`ScopeState`].
    struct ScopeId;
}

/// Internal state for [`Scope`].
struct ScopeState {
    /// A list of callbacks that will be called when the scope is dropped.
    cleanups: Vec<Box<dyn FnOnce()>>,
    /// A list of child scopes owned by this scope. The child scopes will also be dropped when this
    /// scope is dropped.
    child_scopes: Vec<ScopeId>,
    /// A list of signals "owned" by this scope.
    signals: Vec<SignalId>,
    /// A list of effects "owned" by this scope.
    effects: Vec<EffectId>,
    /// A list of context values in this scope.
    context: Vec<Box<dyn Any>>,
    /// The ID of the parent scope, or `None` if this is the root scope.
    parent: Option<ScopeId>,
    root: &'static Root,
}

impl ScopeState {
    /// Create a new `ScopeState` referencing the `root`. This does _not_ insert the `ScopeState`
    /// into the `Root`.
    fn new(root: &'static Root, parent: Option<ScopeId>) -> Self {
        Self {
            child_scopes: Vec::new(),
            cleanups: Vec::new(),
            signals: Vec::new(),
            effects: Vec::new(),
            context: Vec::new(),
            parent,
            root,
        }
    }
}

impl Drop for ScopeState {
    fn drop(&mut self) {
        for cleanup in std::mem::take(&mut self.cleanups) {
            cleanup();
        }
        for child_scope in &self.child_scopes {
            let data = self.root.scopes.borrow_mut().remove(*child_scope);
            drop(data);
        }
        for signal in &self.signals {
            let data = self.root.signals.borrow_mut().remove(*signal);
            drop(data.expect("scope should not be dropped yet"));
        }
        for effect in &self.effects {
            let data = self.root.effects.borrow_mut().remove(*effect);
            drop(data.expect("scope should not be dropped yet"));
        }
    }
}

/// A reference to a reactive scope. This struct is `Copy`, allowing it to be copied into
/// closures without any clones.
///
/// The intended way to access a [`Scope`] is with the [`create_child_scope`] function.
#[derive(Clone, Copy)]
pub struct Scope {
    id: ScopeId,
    root: &'static Root,
}

impl Scope {
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data<T>(self, f: impl FnOnce(&mut ScopeState) -> T) -> T {
        f(&mut self.root.scopes.borrow_mut()[self.id])
    }

    /// Remove the scope from the root and drop it.
    pub fn dispose(self) {
        let data = self.root.scopes.borrow_mut().remove(self.id);
        drop(data.expect("scope should not be dropped yet"));
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope").field("id", &self.id).finish()
    }
}

/// Creates a new reactive root with a top-level [`Scope`]. The returned [`RootHandle`] can be used
/// to [`dispose`](RootHandle::dispose) the root.
///
/// # Example
/// ```rust
/// # use sycamore_reactive3::*;
///
/// create_root(|cx| {
///     let signal = create_signal(cx, 123);
///
///     let child_scope = create_child_scope(cx, move |cx| {
///         // ...
///     });
/// });
/// ```
pub fn create_root(f: impl FnMut(Scope)) -> RootHandle {
    /// An unsafe wrapper around a raw pointer which we promise to never touch, effectively making
    /// it thread-safe.
    struct UnsafeSendPtr<T>(*const T);

    /// We never ever touch the pointer inside so surely this is safe!
    unsafe impl<T> Send for UnsafeSendPtr<T> {}

    /// A static variable to keep on holding to the allocated `Root`s to prevent Miri and Valgrind
    /// from complaining.
    static KEEP_ALIVE: Mutex<Vec<UnsafeSendPtr<Root>>> = Mutex::new(Vec::new());

    let root = Root::default();
    let _ref = Box::leak(Box::new(root));
    KEEP_ALIVE
        .lock()
        .unwrap()
        .push(UnsafeSendPtr(_ref as *const Root));

    let handle = RootHandle { _ref };
    handle.reinitialize(f);
    handle
}

/// Create a child scope.
///
/// Returns the created [`Scope`] which can be used to dispose it.
pub fn create_child_scope(cx: Scope, f: impl FnOnce(Scope)) -> Scope {
    let new = ScopeState::new(cx.root, Some(cx.id));
    let key = cx.root.scopes.borrow_mut().insert(new);
    // Push the new scope onto the child scope list so that it is properly dropped when the parent
    // scope is dropped.
    cx.get_data(|cx| cx.child_scopes.push(key));
    let scope = Scope {
        id: key,
        root: cx.root,
    };
    f(scope);
    scope
}

/// Adds a callback that is called when the scope is destroyed.
///
/// # Example
/// ```rust
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let child_scope = create_child_scope(cx, |cx| {
///     on_cleanup(cx, || {
///         println!("Child scope is being dropped");
///     });
/// });
/// child_scope.dispose(); // Executes the on_cleanup callback.
/// # });
/// ```
pub fn on_cleanup(cx: Scope, f: impl FnOnce() + 'static) {
    cx.get_data(move |cx| cx.cleanups.push(Box::new(f)));
}

/// Batch updates from related signals together and only run effects at the end of the scope.
///
/// Note that this only batches effect updates, not memos. This is because we cannot defer updating
/// of a signal because of methods like [`Signal::update`] which allow direct mutation to the
/// underlying value.
pub fn batch<T>(cx: Scope, f: impl FnOnce() -> T) -> T {
    cx.root.start_batch();
    let ret = f();
    cx.root.end_batch();
    ret
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn cleanup() {
        create_root(|cx| {
            let cleanup_called = create_signal(cx, false);
            let scope = create_child_scope(cx, |cx| {
                on_cleanup(cx, move || {
                    cleanup_called.set(true);
                });
            });
            assert!(!cleanup_called.get());
            scope.dispose();
            assert!(cleanup_called.get());
        });
    }

    #[test]
    fn cleanup_in_effect() {
        create_root(|cx| {
            let trigger = create_signal(cx, ());

            let counter = create_signal(cx, 0);

            create_effect_scoped(cx, move |cx| {
                trigger.track();

                on_cleanup(cx, move || {
                    counter.set(counter.get() + 1);
                });
            });

            assert_eq!(counter.get(), 0);

            trigger.set(());
            assert_eq!(counter.get(), 1);

            trigger.set(());
            assert_eq!(counter.get(), 2);
        });
    }

    #[test]
    fn cleanup_is_untracked() {
        create_root(|cx| {
            let trigger = create_signal(cx, ());

            let counter = create_signal(cx, 0);

            create_effect_scoped(cx, move |cx| {
                counter.set(counter.get_untracked() + 1);

                on_cleanup(cx, move || {
                    trigger.track(); // trigger should not be tracked
                });
            });

            assert_eq!(counter.get(), 1);

            trigger.set(());
            assert_eq!(counter.get(), 1);
        });
    }

    #[test]
    fn batch_updates_effects_at_end() {
        create_root(|cx| {
            let state1 = create_signal(cx, 1);
            let state2 = create_signal(cx, 2);
            let counter = create_signal(cx, 0);
            create_effect(cx, move || {
                counter.set(counter.get_untracked() + 1);
                let _ = state1.get() + state2.get();
            });
            assert_eq!(counter.get(), 1);
            state1.set(2);
            state2.set(3);
            assert_eq!(counter.get(), 3);
            batch(cx, move || {
                state1.set(3);
                assert_eq!(counter.get(), 3);
                state2.set(4);
                assert_eq!(counter.get(), 3);
            });
            assert_eq!(counter.get(), 4);
        });
    }
}

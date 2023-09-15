//! [`Root`] and [`Scope`].

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::fmt;

use slotmap::{new_key_type, SlotMap};

use crate::signals::{Mark, SignalId, SignalState};
use crate::{EffectId, EffectState};

/// The struct managing the state of the reactive system. Only one should be created per running
/// app.
///
/// Often times, this is intended to be leaked to be able to get a `&'static Root`. However, the
/// `Root` is also `dispose`-able, meaning that any resources allocated in this `Root` will get
/// deallocated. Therefore in practice, there should be no memory leak at all except for the `Root`
/// itself. Finally, the `Root` is expected to live for the whole duration of the app so this is
/// not a problem.
pub(crate) struct Root {
    /// A reference to the root scope.
    pub root_scope: Cell<ScopeId>,
    /// All the scopes that have been created in this `Root`.
    pub scopes: RefCell<SlotMap<ScopeId, ScopeState>>,
    /// The current scope that we are running in.
    pub current_scope: Cell<ScopeId>,
    /// All the signals that have been created in this `Root`.
    /// Eventhough signals are stored here, they are created under roots and are destroyed by the
    /// scopes when they themselves are dropped.
    pub signals: RefCell<SlotMap<SignalId, SignalState>>,
    /// ALl the effects that have been created in this `Root`.
    /// Effects are also destroyed by the scopes when they themselves are dropped.
    pub effects: RefCell<SlotMap<EffectId, EffectState>>,
    /// If this is `Some`, that means we are tracking signal accesses.
    pub tracker: RefCell<Option<DependencyTracker>>,
    /// A temporary buffer used in `propagate_updates` to prevent allocating a new Vec every time
    /// it is called.
    pub rev_sorted_buf: RefCell<Vec<SignalId>>,
    /// A list of effects to be run after signal updates are over.
    pub effect_queue: RefCell<Vec<EffectId>>,
    /// Whether we are currently batching signal updatse. If this is true, we do not run
    /// `effect_queue` and instead wait until the end of the batch.
    pub batch: Cell<bool>,
}

thread_local! {
    /// The current reactive root.
    static GLOBAL_ROOT: Cell<Option<&'static Root>> = Cell::new(None);
}

impl Root {
    /// Get the current reactive root. Panics if no root is found.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_global() -> &'static Root {
        GLOBAL_ROOT.with(|root| root.get().expect("no root found"))
    }

    /// Sets the current reactive root. Returns the previous root.
    pub fn set_global(root: Option<&'static Root>) -> Option<&'static Root> {
        GLOBAL_ROOT.with(|r| r.replace(root))
    }

    pub fn new_static() -> &'static Self {
        let this = Self {
            root_scope: Cell::new(ScopeId::default()),
            scopes: RefCell::new(SlotMap::default()),
            current_scope: Cell::new(ScopeId::default()),
            signals: RefCell::new(SlotMap::default()),
            effects: RefCell::new(SlotMap::default()),
            tracker: RefCell::new(None),
            rev_sorted_buf: RefCell::new(Vec::new()),
            effect_queue: RefCell::new(Vec::new()),
            batch: Cell::new(false),
        };
        let _ref = Box::leak(Box::new(this));
        _ref.reinit();
        _ref
    }

    /// Disposes of all the resources held on by this root and resets the state.
    pub fn reinit(&'static self) {
        let _ = self.scopes.take();
        let _ = self.signals.take();
        let _ = self.effects.take();
        let _ = self.tracker.take();
        let _ = self.rev_sorted_buf.take();
        let _ = self.effect_queue.take();
        self.batch.set(false);
        // Create a new top-level scope.
        let root_scope = ScopeState::new(self, None);
        let root_scope_key = self.scopes.borrow_mut().insert(root_scope);
        self.root_scope.set(root_scope_key);
        self.current_scope.set(root_scope_key);
    }

    /// Create a new child scope. Implementation detail for [`create_child_scope`].
    pub fn create_child_scope(&'static self, f: impl FnOnce()) -> Scope {
        // Create a nested scope.
        let parent = self.current_scope.get();
        let scope = ScopeState::new(self, Some(parent));
        let scope_key = self.scopes.borrow_mut().insert(scope);
        self.scopes.borrow_mut()[parent]
            .child_scopes
            .push(scope_key);
        self.current_scope.set(scope_key);
        f();
        self.current_scope.set(parent);
        Scope {
            id: scope_key,
            root: self,
        }
    }

    /// Run the provided closure in a tracked scope. This will detect all the signals that are
    /// accessed and track them in a dependency list.
    pub fn tracked_scope<T>(&self, f: impl FnOnce() -> T) -> (T, DependencyTracker) {
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

        Self::dfs(start_node, &mut self.signals.borrow_mut(), &mut rev_sorted);

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
    pub fn propagate_updates(&'static self, start_node: SignalId) {
        // Set the global root.
        let prev = Root::set_global(Some(self));
        // Propagate any signal updates.
        self.propagate_signal_updates(start_node);
        if !self.batch.get() {
            // Run all the effects that have been queued.
            self.run_effects();
        }
        Root::set_global(prev);
    }

    /// Run depth-first-search on the reactive graph starting at `current`.
    ///
    /// Also resets `changed_in_last_update` and adds a [`Mark::Permanent`] for all signals
    /// traversed.
    fn dfs(
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
            Self::dfs(child, signals, buf);
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
    /// Destroy everything that was created in this scope.
    pub fn dispose(&self) {
        self._ref.reinit();
    }

    /// Runs the closure in the current scope of the root.
    pub fn run_in<T>(&self, f: impl FnOnce() -> T) -> T {
        let prev = Root::set_global(Some(self._ref));
        let ret = f();
        Root::set_global(prev);
        ret
    }
}

/// Tracks signals that are accessed inside a reactive scope.
#[derive(Default)]
pub(crate) struct DependencyTracker {
    /// A list of signals that were accessed.
    pub dependencies: Vec<SignalId>,
}

impl DependencyTracker {
    /// Sets the `dependents` field for all the signals that have been tracked.
    pub fn create_signal_dependency_links(self, root: &Root, dependent: SignalId) {
        for signal in &self.dependencies {
            root.signals.borrow_mut()[*signal]
                .dependents
                .push(dependent);
        }
        // Set the signal dependencies so that it is updated automatically.
        root.signals.borrow_mut()[dependent].dependencies = self.dependencies;
    }

    /// Sets the `effect_dependents` field for all the signals that have been tracked.
    pub fn create_effect_dependency_links(self, root: &Root, dependent: EffectId) {
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
    pub(crate) struct ScopeId;
}

/// Internal state for [`Scope`].
pub(crate) struct ScopeState {
    /// A list of callbacks that will be called when the scope is dropped.
    pub cleanups: Vec<Box<dyn FnOnce()>>,
    /// A list of child scopes owned by this scope. The child scopes will also be dropped when this
    /// scope is dropped.
    pub child_scopes: Vec<ScopeId>,
    /// A list of signals "owned" by this scope.
    pub signals: Vec<SignalId>,
    /// A list of effects "owned" by this scope.
    pub effects: Vec<EffectId>,
    /// A list of context values in this scope.
    pub context: Vec<Box<dyn Any>>,
    /// The ID of the parent scope, or `None` if this is the root scope.
    pub parent: Option<ScopeId>,
    pub root: &'static Root,
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
    pub(crate) id: ScopeId,
    pub(crate) root: &'static Root,
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
/// create_root(|| {
///     let signal = create_signal(123);
///
///     let child_scope = create_child_scope(move || {
///         // ...
///     });
/// });
/// ```
#[must_use = "root should be disposed"]
pub fn create_root(f: impl FnOnce()) -> RootHandle {
    let _ref = Root::new_static();
    #[cfg(not(target_arch = "wasm32"))]
    {
        /// An unsafe wrapper around a raw pointer which we promise to never touch, effectively
        /// making it thread-safe.
        struct UnsafeSendPtr<T>(*const T);
        /// We never ever touch the pointer inside so surely this is safe!
        unsafe impl<T> Send for UnsafeSendPtr<T> {}

        /// A static variable to keep on holding to the allocated `Root`s to prevent Miri and
        /// Valgrind from complaining.
        static KEEP_ALIVE: std::sync::Mutex<Vec<UnsafeSendPtr<Root>>> =
            std::sync::Mutex::new(Vec::new());
        KEEP_ALIVE
            .lock()
            .unwrap()
            .push(UnsafeSendPtr(_ref as *const Root));
    }

    Root::set_global(Some(_ref));
    f();
    Root::set_global(None);
    RootHandle { _ref }
}

/// Create a child scope.
///
/// Returns the created [`Scope`] which can be used to dispose it.
/// TODO: Make sure that the created scope is not a top-level scope.
pub fn create_child_scope(f: impl FnOnce()) -> Scope {
    Root::get_global().create_child_scope(f)
}

/// Adds a callback that is called when the scope is destroyed.
///
/// # Example
/// ```rust
/// # use sycamore_reactive3::*;
/// # create_root(|| {
/// let child_scope = create_child_scope(|| {
///     on_cleanup(|| {
///         println!("Child scope is being dropped");
///     });
/// });
/// child_scope.dispose(); // Executes the on_cleanup callback.
/// # });
/// ```
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    let root = Root::get_global();
    root.scopes.borrow_mut()[root.current_scope.get()]
        .cleanups
        .push(Box::new(f));
}

/// Batch updates from related signals together and only run effects at the end of the scope.
///
/// Note that this only batches effect updates, not memos. This is because we cannot defer updating
/// of a signal because of methods like [`Signal::update`] which allow direct mutation to the
/// underlying value.
pub fn batch<T>(f: impl FnOnce() -> T) -> T {
    let root = Root::get_global();
    root.start_batch();
    let ret = f();
    root.end_batch();
    ret
}

/// Run the passed closure inside an untracked dependency scope.
///
/// See also [`ReadSignal::get_untracked()`].
///
/// # Example
///
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|| {
/// let state = create_signal(1);
/// let double = create_memo(move || untrack(|| state.get() * 2));
/// assert_eq!(double.get(), 2);
///
/// state.set(2);
/// // double value should still be old value because state was untracked
/// assert_eq!(double.get(), 2);
/// # });
/// ```
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    let root = Root::get_global();
    let prev = root.tracker.replace(None);
    let ret = f();
    root.tracker.replace(prev);
    ret
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn cleanup() {
        create_root(|| {
            let cleanup_called = create_signal(false);
            let scope = create_child_scope(|| {
                on_cleanup(move || {
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
        create_root(|| {
            let trigger = create_signal(());

            let counter = create_signal(0);

            create_effect_scoped(move || {
                trigger.track();

                on_cleanup(move || {
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
        create_root(|| {
            let trigger = create_signal(());

            let counter = create_signal(0);

            create_effect_scoped(move || {
                counter.set(counter.get_untracked() + 1);

                on_cleanup(move || {
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
        create_root(|| {
            let state1 = create_signal(1);
            let state2 = create_signal(2);
            let counter = create_signal(0);
            create_effect(move || {
                counter.set(counter.get_untracked() + 1);
                let _ = state1.get() + state2.get();
            });
            assert_eq!(counter.get(), 1);
            state1.set(2);
            state2.set(3);
            assert_eq!(counter.get(), 3);
            batch(move || {
                state1.set(3);
                assert_eq!(counter.get(), 3);
                state2.set(4);
                assert_eq!(counter.get(), 3);
            });
            assert_eq!(counter.get(), 4);
        });
    }
}

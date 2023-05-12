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
// mod iter;
mod effects;
mod memos;
mod signals;

pub use context::*;
pub use effects::*;
// pub use iter::*;
pub use memos::*;
pub use signals::*;

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
        // We take the update callback out because that requires a mut ref and we cannot hold that
        // while running update itself.
        let mut update = self.signals.borrow_mut()[id].update.take();
        let changed = if let Some(update) = &mut update {
            let (changed, tracker) =
                self.tracked_scope(|| update(&mut self.signals.borrow()[id].value.borrow_mut()));
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
        for &effect in &effect_queue {
            self.effects.borrow_mut()[effect].already_run_in_update = false;
        }
        // 2 - Run all the effects.
        for &effect in &effect_queue {
            if !self.effects.borrow_mut()[effect].already_run_in_update {
                self.run_effect_update(effect);
                // Prevent effects from running twice.
                self.effects.borrow_mut()[effect].already_run_in_update = true;
            }
        }
    }

    /// If there are no cyclic dependencies, then the reactive graph is a DAG (Directed Acylic
    /// Graph). We can therefore use DFS to get a topological sorting of all the reactive nodes.
    ///
    /// We then go through every node in this topological sorting and update only those nodes which
    /// have dependencies that were updated. TODO: Is there a way to cut update short if nothing
    /// changed?
    fn propagate_signal_updates(&self, start_node: SignalId) {
        // Avoid allocation by reusing a `Vec` stored in the `Root`.
        let mut rev_sorted = self
            .rev_sorted_buf
            .try_borrow_mut()
            .expect("cannot update a signal inside a memo");
        rev_sorted.clear();

        self.dfs(start_node, &mut rev_sorted);

        for &node in rev_sorted.iter().rev() {
            // Reset value.
            self.signals.borrow_mut()[node].mark = Mark::None;

            // Do not update the starting node since it has already been updated.
            if node == start_node {
                self.signals.borrow_mut()[node].changed_in_last_update = true;
                let dependents =
                    std::mem::take(&mut self.signals.borrow_mut()[node].effect_dependents);
                self.effect_queue.borrow_mut().extend(dependents);
                continue;
            }

            // Check if dependencies are updated.
            let dependencies = std::mem::take(&mut self.signals.borrow_mut()[node].dependencies);
            let any_dep_changed = dependencies
                .iter()
                .any(|dep| self.signals.borrow()[*dep].changed_in_last_update);

            let changed = if any_dep_changed {
                // Both dependencies and dependents have been erased by now.
                self.run_signal_update(node)
            } else {
                false
            };
            self.signals.borrow_mut()[node].changed_in_last_update = changed;

            // If the signal value has changed, add all the effects that depend on it to the effect
            // queue.
            if changed {
                let dependents =
                    std::mem::take(&mut self.signals.borrow_mut()[node].effect_dependents);
                self.effect_queue.borrow_mut().extend(dependents);
            }
        }
    }

    /// Call this if `start_node` has been updated manually. This will automatically update all
    /// signals that depend on `start_node` as well as call any effects as necessary.
    fn propagate_updates(&self, start_node: SignalId) {
        // Propagate any signal updates.
        self.propagate_signal_updates(start_node);
        // Run all the effects that have been queued.
        self.run_effects();
    }

    /// Run depth-first-search on the reactive graph starting at `current`.
    ///
    /// Also resets `changed_in_last_update` and adds a [`Mark::Permanent`] for all signals
    /// traversed.
    fn dfs(&self, current: SignalId, buf: &mut Vec<SignalId>) {
        // Reset value.
        self.signals.borrow_mut()[current].changed_in_last_update = false;

        match self.signals.borrow()[current].mark {
            Mark::Temp => panic!("cylcic reactive dependency detected"),
            Mark::Permanent => return,
            Mark::None => {}
        }
        self.signals.borrow_mut()[current].mark = Mark::Temp;

        let children = std::mem::take(&mut self.signals.borrow_mut()[current].dependents);
        for child in children {
            self.dfs(child, buf);
        }
        self.signals.borrow_mut()[current].mark = Mark::Permanent;
        buf.push(current);
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
            dbg!(signal, "added as dependency of effect", dependent);
            root.signals.borrow_mut()[*signal]
                .effect_dependents
                .push(dependent);
        }
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
            drop(data.expect("child scope should not be dropped yet"));
        }
        for signal in &self.signals {
            let data = self.root.signals.borrow_mut().remove(*signal);
            drop(data.expect("scope should not be dropped yet"));
        }
        for effects in &self.effects {
            let data = self.root.effects.borrow_mut().remove(*effects);
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
pub fn create_child_scope(cx: Scope, mut f: impl FnMut(Scope)) -> Scope {
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

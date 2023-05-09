//! Reactivity runtime for Sycamore.

#![cfg_attr(feature = "nightly", feature(fn_traits))]
#![cfg_attr(feature = "nightly", feature(unboxed_closures))]

use std::cell::{Cell, RefCell};
use std::sync::Mutex;

use signals::{SignalId, SignalState};
use slotmap::{new_key_type, SlotMap};

pub mod memos;
pub mod signals;

#[derive(Default)]
struct Root {
    root_scope: Cell<ScopeId>,
    scopes: RefCell<SlotMap<ScopeId, ScopeState>>,
    signals: RefCell<SlotMap<SignalId, SignalState>>,
    /// If this is `Some`, that means we are tracking signal accesses.
    tracker: RefCell<Option<DependencyTracker>>,
}

impl Root {
    /// Run the provided closure in a tracked scope. This will detect all the signals that are
    /// accessed and track them in a dependency list.
    fn tracked_scope<T>(&self, f: impl FnOnce() -> T) -> (T, DependencyTracker) {
        let prev = self.tracker.replace(Some(DependencyTracker::default()));
        let ret = f();
        (ret, self.tracker.replace(prev).unwrap())
    }

    /// Assuming that there are no circular dependencies, the reactive graph is a DAG (Directed
    /// Acylic Graph). We can therefore do a topological sort on the reactive nodes using Kahn's
    /// algorithm.
    fn topo_sort(&self, mut start: Vec<SignalId>) -> Vec<SignalId> {
        let mut sorted = Vec::new();

        while let Some(node) = start.pop() {
            // Add this node to topo_sorted since we know at this point all dependencies are
            // updated.
            sorted.push(node);
            let dependencies = std::mem::take(&mut self.signals.borrow_mut()[node].dependents);
            for dependent in dependencies {
                // Remove the dependency link in dependent.
                self.signals.borrow_mut()[dependent]
                    .dependencies
                    .retain(|dependency| *dependency != node);
                // Check if there are any remaining dependencies left.
                if self.signals.borrow()[dependent].dependencies.is_empty() {
                    start.push(dependent);
                }
            }
        }

        sorted
    }

    /// Run the update callback of the signal, also recreating any dependencies found by
    /// tracking
    /// signal accesses inside the function.
    ///
    /// # Params
    /// * `root` - The reactive root.
    /// * `id` - The ID associated with this `SignalState`. This is because we are not storing the
    /// `SignalId` inside the state itself.
    pub fn run_update(&self, id: SignalId) {
        // We take the update callback out because that requires a mut ref and we cannot hold that
        // while running update itself.
        let mut update = self.signals.borrow_mut()[id].update.take();
        if let Some(update) = &mut update {
            let (_, tracker) = self.tracked_scope(|| {
                update(&mut self.signals.borrow()[id].value.borrow_mut());
            });

            tracker.create_dependency_links(self, id);
        }
        // Put the update back in.
        self.signals.borrow_mut()[id].update = update;
    }

    /// Call this if `signal` has been updated manually. This will automatically update all signals
    /// that depend on `signal`.
    fn propagate_updates(&self, start: SignalId) {
        let sorted = self.topo_sort(vec![start]);
        // We skip the first node since that's the start node which we don't need to update.
        for node in &sorted[1..] {
            self.run_update(*node);
        }
    }
}

#[derive(Clone, Copy)]
pub struct RootHandle {
    _ref: &'static Root,
}

impl RootHandle {
    /// Reinitializes the [`Root`]. Once the root is reinitialized, nothing from before this call
    /// should reference this `Root`.
    pub fn reinitialize(&self, mut f: impl FnMut(Scope)) {
        // Create an initial scope and call our callback.
        let root_scope = ScopeState::new(self._ref);
        let root_scope_key = self._ref.scopes.borrow_mut().insert(root_scope);
        self._ref.root_scope.set(root_scope_key);

        let cx = Scope {
            id: root_scope_key,
            root: self._ref,
        };
        f(cx);
    }

    pub fn dispose(&self) {
        self.reinitialize(|_| {})
    }
}

#[derive(Default)]
struct DependencyTracker {
    /// A list of signals that were accessed.
    dependencies: Vec<SignalId>,
}

impl DependencyTracker {
    /// Sets the `dependents` field for all the signals that have been tracked.
    fn create_dependency_links(self, root: &Root, dependent: SignalId) {
        for signal in &self.dependencies {
            root.signals.borrow_mut()[*signal]
                .dependents
                .push(dependent);
        }
        // Set the signal dependencies so that it is updated automatically.
        root.signals.borrow_mut()[dependent].dependencies = self.dependencies;
    }
}

new_key_type! { struct ScopeId; }

struct ScopeState {
    /// A list of callbacks that will be called when the scope is dropped.
    cleanups: Vec<Box<dyn FnOnce()>>,
    /// A list of child scopes owned by this scope. The child scopes will also be dropped when this
    /// scope is dropped.
    child_scopes: Vec<ScopeId>,
    /// A list of signals "owned" by this scope.
    signals: Vec<SignalId>,
    root: &'static Root,
}

impl ScopeState {
    fn new(root: &'static Root) -> Self {
        Self {
            child_scopes: Vec::new(),
            cleanups: Vec::new(),
            signals: Vec::new(),
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
    }
}

/// Represents a reactive scope.
#[derive(Clone, Copy)]
pub struct Scope {
    id: ScopeId,
    root: &'static Root,
}

impl Scope {
    pub(crate) fn get_data<T>(self, f: impl FnOnce(&mut ScopeState) -> T) -> T {
        f(&mut self.root.scopes.borrow_mut()[self.id])
    }

    pub fn dispose(self) {
        let data = self.root.scopes.borrow_mut().remove(self.id);
        drop(data.expect("scope should not be dropped yet"));
    }
}

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

pub fn create_child_scope(cx: Scope, mut f: impl FnMut(Scope)) -> Scope {
    let new = ScopeState::new(cx.root);
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

pub fn on_cleanup(cx: Scope, f: impl FnOnce() + 'static) {
    cx.get_data(move |cx| cx.cleanups.push(Box::new(f)));
}

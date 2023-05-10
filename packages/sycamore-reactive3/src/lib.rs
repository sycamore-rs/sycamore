//! Reactivity runtime for Sycamore.

#![cfg_attr(feature = "nightly", feature(fn_traits))]
#![cfg_attr(feature = "nightly", feature(unboxed_closures))]

use std::cell::{Cell, RefCell};
use std::sync::Mutex;

use signals::{Mark, SignalId, SignalState};
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
    /// A temporary buffer used in `propagate_updates` to prevent allocating a new Vec every time
    /// it is called.
    rev_sorted_buf: RefCell<Vec<SignalId>>,
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
    fn run_update(&self, id: SignalId) -> bool {
        // We take the update callback out because that requires a mut ref and we cannot hold that
        // while running update itself.
        let mut update = self.signals.borrow_mut()[id].update.take();
        let changed = if let Some(update) = &mut update {
            let (changed, tracker) =
                self.tracked_scope(|| update(&mut self.signals.borrow()[id].value.borrow_mut()));
            tracker.create_dependency_links(self, id);
            changed
        } else {
            false
        };
        // Put the update back in.
        self.signals.borrow_mut()[id].update = update;
        changed
    }

    /// Call this if `start_node` has been updated manually. This will automatically update all
    /// signals that depend on `start_node`.
    ///
    /// If there are no cyclic dependencies, then the reactive graph is a DAG (Directed Acylic
    /// Graph). We can therefore use DFS to get a topological sorting of all the reactive nodes.
    ///
    /// We then go through every node in this topological sorting and update only those nodes which
    /// have dependencies that were updated. TODO: Is there a way to cut update short if nothing
    /// changed?
    fn propagate_updates(&self, start_node: SignalId) {
        let mut rev_sorted = self.rev_sorted_buf.borrow_mut();
        rev_sorted.clear();

        self.dfs(start_node, &mut rev_sorted);

        for &node in rev_sorted.iter().rev() {
            // Reset value.
            self.signals.borrow_mut()[node].mark = Mark::None;

            // Do not update the starting node since it has already been updated.
            if node == start_node {
                self.signals.borrow_mut()[node].changed_in_last_update = true;
                continue;
            }

            // Check if dependencies are updated.
            let dependencies = std::mem::take(&mut self.signals.borrow_mut()[node].dependencies);
            let any_dep_changed = dependencies
                .iter()
                .any(|dep| self.signals.borrow()[*dep].changed_in_last_update);

            let changed = if any_dep_changed {
                // Both dependencies and dependents have been erased by now.
                self.run_update(node)
            } else {
                false
            };
            self.signals.borrow_mut()[node].changed_in_last_update = changed;
        }
    }

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
    #[cfg_attr(debug_assertions, track_caller)]
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

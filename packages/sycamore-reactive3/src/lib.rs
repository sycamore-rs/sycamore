//! Reactivity runtime for Sycamore.

use std::sync::Mutex;

use signals::{Signal, SignalKey};
use slotmap::{new_key_type, SlotMap};

pub mod signals;

struct Root {
    root_scope: Mutex<ScopeKey>,
    scopes: Mutex<SlotMap<ScopeKey, ScopeData>>,
}

#[derive(Clone, Copy)]
pub struct RootHandle {
    _ref: &'static Root,
}

impl RootHandle {
    /// Resets the [`Root`]. Once the root is disposed, nothing referencing this `Root` should be
    /// called.
    pub fn reinitialize(&self, mut f: impl FnMut(Scope)) {
        // Create an initial scope and call our callback.
        let root_scope = ScopeData::new(self._ref);
        let root_scope_key = self._ref.scopes.lock().unwrap().insert(root_scope);
        *self._ref.root_scope.lock().unwrap() = root_scope_key;

        let cx = Scope {
            key: root_scope_key,
            root: self._ref,
        };
        f(cx);
    }
}

new_key_type! { struct ScopeKey; }

struct ScopeData {
    child_scopes: Vec<ScopeKey>,
    signals: SlotMap<SignalKey, Signal>,
    root: &'static Root,
}

impl ScopeData {
    fn new(root: &'static Root) -> Self {
        Self {
            child_scopes: Vec::new(),
            signals: SlotMap::default(),
            root,
        }
    }
}

/// Represents a reactive scope.
pub struct Scope {
    key: ScopeKey,
    root: &'static Root,
}

pub fn create_root(f: impl FnMut(Scope)) -> RootHandle {
    /// A static variable to keep on holding to the allocated `Root`s to prevent Miri and Valgrind
    /// from complaining.
    static KEEP_ALIVE: Mutex<Vec<&'static Root>> = Mutex::new(Vec::new());

    let root = Root {
        root_scope: Default::default(),
        scopes: Default::default(),
    };
    let _ref = Box::leak(Box::new(root));
    KEEP_ALIVE.lock().unwrap().push(_ref);

    let handle = RootHandle { _ref };
    handle.reinitialize(f);
    handle
}

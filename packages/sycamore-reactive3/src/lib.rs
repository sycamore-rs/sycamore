//! Reactivity runtime for Sycamore.

use std::cell::{Cell, RefCell};
use std::sync::Mutex;

use signals::{SignalData, SignalKey};
use slotmap::{new_key_type, SlotMap};

pub mod signals;

struct Root {
    root_scope: Cell<ScopeKey>,
    scopes: RefCell<SlotMap<ScopeKey, ScopeData>>,
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
        let root_scope = ScopeData::new(self._ref);
        let root_scope_key = self._ref.scopes.borrow_mut().insert(root_scope);
        self._ref.root_scope.set(root_scope_key);

        let cx = Scope {
            key: root_scope_key,
            root: self._ref,
        };
        f(cx);
    }

    pub fn dispose(&self) {
        self.reinitialize(|_| {})
    }
}

new_key_type! { struct ScopeKey; }

struct ScopeData {
    child_scopes: Vec<ScopeKey>,
    signals: SlotMap<SignalKey, SignalData>,
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
#[derive(Clone, Copy)]
pub struct Scope {
    key: ScopeKey,
    root: &'static Root,
}

impl Scope {
    fn get_data<T>(self, f: impl FnOnce(&mut ScopeData) -> T) -> T {
        f(&mut self.root.scopes.borrow_mut()[self.key])
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

    let root = Root {
        root_scope: Default::default(),
        scopes: Default::default(),
    };
    let _ref = Box::leak(Box::new(root));
    KEEP_ALIVE
        .lock()
        .unwrap()
        .push(UnsafeSendPtr(_ref as *const Root));

    let handle = RootHandle { _ref };
    handle.reinitialize(f);
    handle
}

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;

use slab::Slab;

thread_local! {
    /// The current [`ReactiveScope`] of the current thread and the key to [`SCOPES`].
    pub(crate) static CURRENT_SCOPE: RefCell<Option<(ReactiveScope, usize)>> = RefCell::new(None);
    /// A slab of all [`ReactiveScope`]s that are currently valid in the current thread.
    pub(crate) static SCOPES: RefCell<Slab<ReactiveScope>> = RefCell::new(Slab::new());
}

#[derive(Clone)]
pub struct ReactiveScope {
    /// Unique identifier for this scope.
    id: usize,
    /// The [`ReactiveScope`] owns all signals that are created within the scope.
    signals: Rc<RefCell<Vec<Box<dyn Any>>>>,
}

impl ReactiveScope {
    /// Returns an incrementing unique identifier for a new [`ReactiveScope`].
    fn get_next_id() -> usize {
        thread_local! {
            static NEXT_ID: Cell<usize> = Cell::new(0);
        }

        NEXT_ID.with(|next_id| {
            let id = next_id.get();
            next_id.set(id + 1);
            id
        })
    }

    /// Get the reactive scope's id.
    pub(crate) fn id(&self) -> usize {
        self.id
    }

    /// Get a reference to the reactive scope's signals.
    pub fn signals(&self) -> &Rc<RefCell<Vec<Box<dyn Any>>>> {
        &self.signals
    }
}

/// Wrapper around [`ReactiveScope`] that will remove the [`ReactiveScope`] from the valid scope
/// slab when it is dropped.
pub struct RootScope {
    key: usize,
    _scope: ReactiveScope,
}

impl Drop for RootScope {
    fn drop(&mut self) {
        remove_scope(self.key);
    }
}

fn insert_scope(scope: ReactiveScope) -> usize {
    SCOPES.with(|scopes| scopes.borrow_mut().insert(scope))
}

fn remove_scope(key: usize) {
    SCOPES.with(|scopes| scopes.borrow_mut().remove(key));
}

pub fn create_root_scope(f: impl FnOnce()) -> RootScope {
    CURRENT_SCOPE.with(|current_scope| {
        let scope = ReactiveScope {
            id: ReactiveScope::get_next_id(),
            signals: Rc::new(RefCell::new(Vec::new())),
        };
        let key = insert_scope(scope.clone());
        let outer = mem::replace(&mut *current_scope.borrow_mut(), Some((scope, key)));
        f();
        let (scope, key) = mem::replace(&mut *current_scope.borrow_mut(), outer).unwrap();
        RootScope { key, _scope: scope }
    })
}

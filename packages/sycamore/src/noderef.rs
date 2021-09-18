//! References to nodes in templates.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::generic_node::GenericNode;
use std::any::Any;

/// A reference to a [`GenericNode`].
#[derive(Clone, PartialEq, Eq)]
pub struct NodeRef<G: GenericNode>(Rc<RefCell<Option<G>>>);

impl<G: GenericNode + Any> NodeRef<G> {
    /// Creates an empty [`NodeRef`].
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    /// Gets the T stored inside the [`NodeRef`].
    ///
    /// # Panics
    /// Panics if the [`NodeRef`] is not set yet or is the wrong type.
    ///
    /// For a non panicking version, see [`NodeRef::try_get`].
    #[track_caller]
    pub fn get<T: GenericNode>(&self) -> T {
        self.try_get().expect_throw("NodeRef is not set")
    }

    /// Tries to get the T stored inside the [`NodeRef`] or `None` if it is not yet set or
    /// the wrong type.
    ///
    /// For a panicking version, see [`NodeRef::get`].
    pub fn try_get<T: GenericNode>(&self) -> Option<T> {
        let obj = self.0.borrow();
        (obj.as_ref()? as &dyn Any).downcast_ref().cloned()
    }

    /// Gets the raw [`GenericNode`] stored inside the [`NodeRef`].
    ///
    /// # Panics
    /// Panics if the [`NodeRef`] is not set yet.
    ///
    /// For a non panicking version, see [`NodeRef::try_get_raw`].
    #[track_caller]
    pub fn get_raw(&self) -> G {
        self.try_get().expect_throw("NodeRef is not set")
    }

    /// Tries to get the raw [`GenericNode`] stored inside the [`NodeRef`] or `None` if it is
    /// not yet set.
    ///
    /// For a panicking version, see [`NodeRef::get`].
    pub fn try_get_raw(&self) -> Option<G> {
        self.0.borrow().clone()
    }

    /// Sets the [`NodeRef`] with the specified [`GenericNode`].
    pub fn set(&self, node: G) {
        *self.0.borrow_mut() = Some(node);
    }
}

impl<G: GenericNode> Default for NodeRef<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: GenericNode> fmt::Debug for NodeRef<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NodeRef").field(&self.0.borrow()).finish()
    }
}

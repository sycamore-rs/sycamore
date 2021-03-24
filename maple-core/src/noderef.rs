use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use web_sys::Node;

/// A reference to a [`Node`] in the DOM.
#[derive(Clone, PartialEq, Eq)]
pub struct NodeRef(Rc<RefCell<Option<Node>>>);

impl NodeRef {
    /// Creates an empty [`NodeRef`].
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    /// Gets the [`Node`] stored inside the [`NodeRef`].
    ///
    /// # Panics
    /// Panics if the [`NodeRef`] is not set yet.
    ///
    /// For a non panicking version, see [`NodeRef::try_get`].
    pub fn get(&self) -> Node {
        self.try_get().expect("NodeRef is not set")
    }

    /// Tries to get the [`Node`] stored inside the [`NodeRef`] or `None` if it is not yet set.
    ///
    /// For a panicking version, see [`NodeRef::get`].
    pub fn try_get(&self) -> Option<Node> {
        self.0.borrow().clone()
    }

    /// Sets the [`NodeRef`] with the specified [`Node`].
    pub fn set(&self, node: Node) {
        *self.0.borrow_mut() = Some(node);
    }
}

impl Default for NodeRef {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NodeRef").field(&self.0.borrow()).finish()
    }
}

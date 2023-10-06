//! Reactive nodes.

use std::any::Any;

use slotmap::new_key_type;
use smallvec::SmallVec;

use crate::{untrack_in_scope, Root};

new_key_type! {
    pub struct NodeId;
}

/// A reactive node inside the reactive grpah.
pub(crate) struct ReactiveNode {
    /// Value of the node, if any. If this node is a signal, should have a value.
    pub value: Option<Box<dyn Any>>,
    /// Callback when node needs to be updated. Returns a `bool` indicating whether the value has
    /// changed or not.
    #[allow(clippy::type_complexity)]
    pub callback: Option<Box<dyn FnMut(&mut Box<dyn Any>) -> bool>>,
    /// Nodes that are owned by this node.
    pub children: Vec<NodeId>,
    /// The parent of this node (i.e. the node that owns this node). If there is no parent, then
    /// this field is set to the "null" key.
    pub parent: NodeId,
    /// Nodes that depend on this node.
    pub dependents: Vec<NodeId>,
    /// Nodes that this node depends on.
    pub dependencies: SmallVec<[NodeId; 1]>,
    /// Callbacks called when node is disposed.
    pub cleanups: Vec<Box<dyn FnOnce()>>,
    /// Context values stored in this node.
    pub context: Vec<Box<dyn Any>>,
    /// Used for keeping track of dirty state of node value.
    pub state: NodeState,
    /// Used for DFS traversal of the reactive graph.
    pub mark: Mark,
    /// Keep track of where the signal was created for diagnostics.
    #[cfg(debug_assertions)]
    pub created_at: &'static std::panic::Location<'static>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeState {
    Clean,
    Dirty,
}

/// A mark used for DFS traversal of the reactive graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mark {
    /// Mark when DFS reaches node.
    Temp,
    /// Mark when DFS is done with node.
    Permanent,
    /// No mark.
    None,
}

#[derive(Clone, Copy)]
pub struct NodeHandle(pub(crate) NodeId, pub(crate) &'static Root);

impl NodeHandle {
    pub fn dispose(self) {
        // Dispose children first since this node could be referenced in a cleanup.
        self.dispose_children();
        let mut nodes = self.1.nodes.borrow_mut();
        // Release memory.
        if let Some(this) = nodes.remove(self.0) {
            // Remove self from all dependencies.
            for dependent in this.dependents {
                // dependent might have been removed if it is a child node.
                if let Some(dependent) = nodes.get_mut(dependent) {
                    dependent.dependencies.retain(|&mut id| id != self.0);
                }
            }
        }
    }

    pub fn dispose_children(self) {
        // If node is already disposed, do nothing.
        if self.1.nodes.borrow().get(self.0).is_none() {
            return;
        }
        let cleanup = std::mem::take(&mut self.1.nodes.borrow_mut()[self.0].cleanups);
        let children = std::mem::take(&mut self.1.nodes.borrow_mut()[self.0].children);

        // Run the cleanup functions in an untracked scope so that we don't track dependencies.
        untrack_in_scope(
            move || {
                for cb in cleanup {
                    cb();
                }
            },
            self.1,
        );
        for child in children {
            Self(child, self.1).dispose();
        }
    }

    pub fn run_in<T>(&self, f: impl FnOnce() -> T) -> T {
        let root = self.1;
        let prev_root = Root::set_global(Some(root));
        let prev_node = root.current_node.replace(self.0);
        let ret = f();
        root.current_node.set(prev_node);
        Root::set_global(prev_root);
        ret
    }
}

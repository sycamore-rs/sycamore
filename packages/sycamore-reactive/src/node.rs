//! Reactive nodes.

use std::any::Any;

use slotmap::new_key_type;

use crate::Root;

new_key_type! {
    pub struct NodeId;
}

/// A reactive node inside the reactive grpah.
pub(crate) struct ReactiveNode {
    /// Value of the node, if any. If this node is a signal, should have a value.
    pub value: Option<Box<dyn Any>>,
    /// Callback when node needs to be updated.
    pub callback: Option<Box<dyn FnMut(&mut Box<dyn Any>) -> bool>>,
    /// Nodes that are owned by this node.
    pub children: Vec<NodeId>,
    /// The parent of this node (i.e. the node that owns this node). If there is no parent, then
    /// this field is set to the "null" key.
    pub parent: NodeId,
    /// Nodes that depend on this node.
    pub dependents: Vec<NodeId>,
    /// Nodes that this node depends on.
    pub dependencies: Vec<NodeId>,
    /// Callbacks called when node is disposed.
    pub cleanups: Vec<Box<dyn FnOnce()>>,
    /// Context values stored in this node.
    pub context: Vec<Box<dyn Any>>,
    /// Used for keeping track of dirty state of node value.
    pub state: NodeState,
    /// Used for DFS traversal of the reactive graph.
    pub mark: Mark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeState {
    Changed,
    Unchanged,
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

#[derive(Debug)]
pub struct NodeHandle(pub(crate) NodeId);

impl NodeId {
    pub fn dispose(self) {
        let root = Root::global();
        self.dispose_children();
        root.nodes.borrow_mut().remove(self);
    }

    pub fn dispose_children(self) {
        let root = Root::global();
        let cleanup = std::mem::take(&mut root.nodes.borrow_mut()[self].cleanups);
        let children = std::mem::take(&mut root.nodes.borrow_mut()[self].children);
        for cb in cleanup {
            cb();
        }
        for child in children {
            child.dispose();
        }
    }
}

impl NodeHandle {
    pub fn dispose(self) {
        self.0.dispose();
    }
}

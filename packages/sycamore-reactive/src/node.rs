//! Reactive nodes.

use std::any::Any;

use slotmap::new_key_type;
use smallvec::SmallVec;

use crate::{Root, untrack_in_scope};

new_key_type! {
    pub(crate) struct NodeId;
}

/// A reactive node inside the reactive graph.
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
    #[allow(dead_code)]
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

/// A handle to a reactive node (signal, memo, effect) that lets you run further tasks in it or
/// manually dispose it.
#[derive(Clone, Copy)]
pub struct NodeHandle(pub(crate) NodeId, pub(crate) &'static Root);

impl NodeHandle {
    /// Disposes the node that is being referenced by this handle. If the node has already been
    /// disposed, this does nothing.
    ///
    /// Automatically calls [`NodeHandle::dispose_children`].
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
            // Remove self from all dependents.
            for dependency in this.dependencies {
                // dependency might have been removed if it is a child node.
                if let Some(dependency) = nodes.get_mut(dependency) {
                    dependency.dependents.retain(|&id| id != self.0);
                }
            }
        }
    }

    /// Dispose all the children of the node but not the node itself.
    ///
    /// Also calls cleanup callbacks and removes context values.
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

        // Clear context values.
        self.1.nodes.borrow_mut()[self.0].context.clear();
    }

    /// Run a closure under this reactive node.
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn dispose_cleans_up_downstream_dependents() {
        let _ = create_root(|| {
            let signal = create_signal(0);
            let memo = create_memo(move || signal.get() + 1);

            let root = Root::global();

            assert_eq!(root.nodes.borrow()[signal.id].dependents, vec![memo.id]);
            assert_eq!(
                root.nodes.borrow()[memo.id].dependencies.to_vec(),
                vec![signal.id]
            );

            signal.dispose();
            assert!(root.nodes.borrow().get(signal.id).is_none());
            assert_eq!(root.nodes.borrow()[memo.id].dependencies.to_vec(), vec![]);
        });
    }

    #[test]
    fn dispose_cleans_up_upstream_dependencies() {
        let _ = create_root(|| {
            let signal = create_signal(0);
            let memo = create_memo(move || signal.get() + 1);

            let root = Root::global();
            memo.dispose();
            assert!(root.nodes.borrow().get(memo.id).is_none());
            assert_eq!(root.nodes.borrow()[signal.id].dependents, vec![]);
        });
    }
}

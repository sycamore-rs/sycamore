//! Reactive nodes.

use std::any::Any;

use slotmap::new_key_type;

use crate::Root;

new_key_type! {
    pub struct NodeId;
}

pub(crate) struct ReactiveNode {
    pub ty: ReactiveNodeType,
    pub value: Option<Box<dyn Any>>,
    pub children: Vec<NodeId>,
}

pub(crate) enum ReactiveNodeType {
    Signal,
    Memo { cb: Box<dyn Fn() -> Box<dyn Any>> },
    Effect { cb: Box<dyn FnMut()> },
    Updating,
}

impl Root {
    pub fn get_value_clone<T: Clone + 'static>(&self, id: NodeId) -> Option<T> {
        self.nodes.borrow().get(id).map(|x| {
            x.value
                .as_ref()
                .unwrap()
                .downcast_ref::<T>()
                .unwrap()
                .clone()
        })
    }
}

//! General utilities for working with attributes.

use sycamore_reactive::Scope;

use crate::generic_node::GenericNode;

/// An attribute that can be applied to a node.
/// These can be implemented only for a specific type that implements [`GenericNode`], rather than
/// for all types that implement [`GenericNode`].
pub trait ApplyAttr<'a, G: GenericNode, T> {
    fn apply(self, cx: Scope<'a>, el: &G, value: T);
}

/// An attribute that can be applied dynamically to a node.
pub trait ApplyAttrDyn<'a, G: GenericNode, T> {
    fn apply(self, cx: Scope<'a>, el: &G, value: Box<dyn FnMut() -> T + 'a>);
}

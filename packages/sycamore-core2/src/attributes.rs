//! General utilities for working with attributes.

use sycamore_reactive::Scope;

use crate::elements::TypedElement;
use crate::generic_node::GenericNode;

/// An attribute that can be applied to a node.
/// These can be implemented only for a specific type that implements [`GenericNode`], rather than
/// for all types that implement [`GenericNode`].
pub trait ApplyAttr<'a, G: GenericNode, T, E: TypedElement<G>> {
    /// Whether this attribute needs to be hydrated. For example, this is `true` for event handlers
    /// because they cannot be serialized into static HTML.
    ///
    /// Note that [`ApplyAttrDyn`] is always hydrated.
    const NEEDS_HYDRATE: bool = false;

    fn apply(self, cx: Scope<'a>, el: &G, value: T);
}

/// An attribute that can be applied dynamically to a node.
pub trait ApplyAttrDyn<'a, G: GenericNode, T, E: TypedElement<G>> {
    fn apply_dyn(self, cx: Scope<'a>, el: &G, value: Box<dyn FnMut() -> T + 'a>);
}

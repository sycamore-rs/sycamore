//! General utilities for working with elements.

use std::fmt;

use sycamore_reactive::Scope;

use crate::attributes::{ApplyAttr, ApplyAttrDyn};
use crate::generic_node::GenericNode;

/// A marker trait that is implemented by elements that can be used with the specified
/// [`GenericNode`] rendering backend.
pub trait TypedElement<G: GenericNode> {}

/// A struct for keeping track for element-building.
pub struct ElementBuilder<'a, G: GenericNode, E: TypedElement<G>> {
    cx: Scope<'a>,
    element: G,
    _marker: std::marker::PhantomData<E>,
}

impl<'a, G: GenericNode, E: TypedElement<G>> ElementBuilder<'a, G, E> {
    /// Creates a new [`ElementBuilder`] with the specified element.
    ///
    /// The input is untyped, so it is possible to construct an `ElementBuilder` of any element type
    /// with this method, regardless of the actual type of the underlying element.
    pub fn new(cx: Scope<'a>, element: G) -> Self {
        Self {
            cx,
            element,
            _marker: std::marker::PhantomData,
        }
    }

    /// Consumes the [`ElementBuilder`] and returns the element.
    pub fn into_element(self) -> G {
        self.element
    }

    /// Applies an attribute to the element.
    pub fn with<Value, Attr: ApplyAttr<'a, G, Value, E>>(self, attr: Attr, value: Value) -> Self {
        attr.apply(self.cx, &self.element, value);
        self
    }

    /// Applies an attribute to the element.
    pub fn with_dyn<Value, Attr: ApplyAttrDyn<'a, G, Value, E>>(
        self,
        attr: Attr,
        value: impl FnMut() -> Value + 'a,
    ) -> Self {
        attr.apply_dyn(self.cx, &self.element, Box::new(value));
        self
    }

    // pub fn children()
}

impl<'a, G: GenericNode, E: TypedElement<G>> fmt::Debug for ElementBuilder<'a, G, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementBuilder")
            .field("element", &self.element)
            .finish()
    }
}

/// A marker type that can represent any element whatsoever. Should be used together with
/// [`ElementBuilder`].
#[derive(Debug)]
pub struct AnyElement;

impl<G: GenericNode> TypedElement<G> for AnyElement {}

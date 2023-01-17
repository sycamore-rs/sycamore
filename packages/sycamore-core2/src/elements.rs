//! General utilities for working with elements.

use std::fmt;

use sycamore_reactive::Scope;

use crate::attributes::{ApplyAttr, ApplyAttrDyn};
use crate::generic_node::GenericNode;
use crate::render;
use crate::view::{ToView, View};

/// A marker trait that is implemented by elements that can be used with the specified
/// [`GenericNode`] rendering backend.
pub trait TypedElement<G: GenericNode> {}

/// Builder-pattern for elements.
pub struct ElementBuilder<'a, G: GenericNode, E: TypedElement<G>> {
    cx: Scope<'a>,
    /// The element that is being built.
    el: G,
    /// Whether the element is dynamic. In SSR, an extra `data-hk` attribute is
    /// added. In client-side hydration, all elements that are not dynamic ignored.
    is_dyn: bool,
    _marker: std::marker::PhantomData<E>,
}

impl<'a, G: GenericNode, E: TypedElement<G>> ElementBuilder<'a, G, E> {
    /// Creates a new [`ElementBuilder`] with the specified element.
    ///
    /// The input is untyped, so it is possible to construct an `ElementBuilder` of any element type
    /// with this method, regardless of the actual type of the underlying element.
    pub fn new(cx: Scope<'a>, el: G) -> Self {
        Self {
            cx,
            el,
            is_dyn: false,
            _marker: std::marker::PhantomData,
        }
    }

    /// Consumes the [`ElementBuilder`] and returns the element.
    pub fn finish(mut self) -> G {
        self.el.finish_element(self.cx, self.is_dyn);
        self.el
    }

    /// Consumes the [`ElementBuilder`] and returns a newly constructed [`View`].
    pub fn view(self) -> View<G> {
        View::new_node(self.finish())
    }

    /// Mark this element as dynamic. This sets the `is_dyn` flag to true.
    fn mark_dyn(&mut self) {
        self.is_dyn = true;
    }

    /// Applies an attribute to the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>(cx: Scope) -> View<G> {
    /// p(cx)
    ///     .with(attr::class, "hello-text")
    ///     .child("Hello, World!")
    /// # .view() }
    /// ```
    pub fn with<Value, Attr: ApplyAttr<'a, G, Value, E>>(self, attr: Attr, value: Value) -> Self {
        attr.apply(self.cx, &self.el, value);
        self
    }

    /// Applies an attribute to the element.
    pub fn with_dyn<Value, Attr: ApplyAttrDyn<'a, G, Value, E>>(
        mut self,
        attr: Attr,
        value: impl FnMut() -> Value + 'a,
    ) -> Self {
        self.mark_dyn();
        attr.apply_dyn(self.cx, &self.el, Box::new(value));
        self
    }

    /// Insert a child node under this element. The inserted child is static by default.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>(cx: Scope) -> View<G> {
    /// div(cx).child(
    ///     h1().child("I am a child")
    /// )
    /// # .view() }
    /// ```
    pub fn child(mut self, c: impl ToView<G>) -> Self {
        let view = c.to_view(self.cx);
        if !view.is_node() {
            self.mark_dyn();
        }
        render::insert(self.cx, &self.el, view, None, None, true);

        self
    }

    /// Cast this [`ElementBuilder`] to a type-erased [`ElementBuilder`].
    pub fn as_any(self) -> ElementBuilder<'a, G, AnyElement> {
        ElementBuilder {
            cx: self.cx,
            el: self.el,
            is_dyn: self.is_dyn,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, G: GenericNode, E: TypedElement<G>> fmt::Debug for ElementBuilder<'a, G, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementBuilder")
            .field("element", &self.el)
            .finish()
    }
}

/// A marker type that can represent any element whatsoever. Should be used together with
/// [`ElementBuilder`].
#[derive(Debug)]
pub struct AnyElement;

impl<G: GenericNode> TypedElement<G> for AnyElement {}

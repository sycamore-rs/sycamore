//! General utilities for working with elements.

use std::fmt;

use sycamore_reactive::{create_effect, Scope};

use crate::generic_node::GenericNode;
use crate::view::{ToView, View};

/// A marker trait that is implemented by elements that can be used with the specified
/// [`GenericNode`] rendering backend.
pub trait TypedElement<G: GenericNode>: Clone + Sized {
    /// Create a new element from the specified node.
    fn from_node(node: G) -> Self;
    /// Get a reference to the underlying [`WebNode`].
    fn as_node(&self) -> &G;
    /// Consume this element and return the untyped [`WebNode`].
    fn into_node(self) -> G;
    /// Create a new [`View`] from this element.
    fn build(&self) -> View<G> {
        View::new_node(self.as_node().clone())
    }
}

/// A trait that is specifically implemented by [`ElementBuilder`] so that we can access the
/// underlying node when implementing a new trait for [`ElementBuilder`].
pub trait AsNode<G: GenericNode> {
    /// Get a reference to the underlying [`GenericNode`].
    fn as_node(&self) -> &G;
}

/// Builder struct for elements.
///
/// Generally, you can get an [`ElementBuilder`] by calling the element's method.
pub struct ElementBuilder<'a, E: TypedElement<G>, G: GenericNode> {
    /// Hold on to the `Scope` so that we can use it without requesting it as a parameter all the
    /// time.
    cx: Scope<'a>,
    /// The element that is being built.
    el: E,
    /// Whether the element is dynamic. In SSR, an extra `data-hk` attribute is
    /// added. In client-side hydration, all elements that are not dynamic ignored.
    is_dyn: bool,
    _marker: std::marker::PhantomData<G>,
}

impl<'a, E: TypedElement<G>, G: GenericNode> ElementBuilder<'a, E, G> {
    /// Creates a new [`ElementBuilder`] with the specified element.
    ///
    /// The input is untyped, so it is possible to construct an `ElementBuilder` of any element type
    /// with this method, regardless of the actual type of the underlying element.
    pub fn from_element(cx: Scope<'a>, el: E) -> Self {
        Self {
            cx,
            el,
            is_dyn: false,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn new(cx: Scope<'a>, f: impl Fn() -> G) -> Self {
        let el = G::get_next_element(cx, f);
        let el = E::from_node(el);
        Self {
            cx,
            el,
            is_dyn: false,
            _marker: std::marker::PhantomData,
        }
    }

    /// Consumes the [`ElementBuilder`] and returns the element.
    pub fn finish(self) -> G {
        let mut node = self.el.into_node();
        node.finish_element(self.cx, self.is_dyn);
        node
    }

    /// Consumes the [`ElementBuilder`] and returns a newly constructed [`View`].
    pub fn view(self) -> View<G> {
        View::new_node(self.finish())
    }

    /// Mark this element as dynamic. This sets the `is_dyn` flag to true.
    ///
    /// This is mostly used internally to signal when an element should be flagged for hydration.
    pub fn mark_dyn(&mut self) {
        self.is_dyn = true;
    }

    /// Modify the element dynamically.
    ///
    /// For instance, you can use this to dynamically set an attribute value.
    pub fn dynamic<U>(mut self, mut f: impl FnMut(Self) -> U + 'a) -> Self
    where
        E: 'a,
    {
        self.mark_dyn();
        let cloned = self.clone();
        create_effect(self.cx, move || {
            let _ = f(cloned.clone());
        });
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
    ///     h1(cx).child("I am a child")
    /// )
    /// # .view() }
    /// ```
    pub fn child<S>(mut self, c: impl ElementBuilderOrView<G, S>) -> Self {
        let view = c.into_view(self.cx);
        if !view.is_node() {
            self.mark_dyn();
        }
        self.el.as_node().builder_insert(self.cx, view);

        self
    }

    /// Get the [`Scope`] of this [`ElementBuilder`].
    pub fn cx(&self) -> Scope<'a> {
        self.cx
    }

    /// Get the underlying [`GenericNode`].
    ///
    /// This should not be used to get the node out of the builder. Instead, prefer to use
    /// `.finish()` or `.view()` instead.
    pub fn as_node(&self) -> &G {
        self.el.as_node()
    }

    /// Spread something onto this element. This uses the [`Spread`] trait behind the scenes.
    pub fn spread<S>(self, value: S) -> Self
    where
        S: Spread<'a, E, G>,
    {
        value.spread(self.cx, self.as_node());
        self
    }
}

impl<'a, E: TypedElement<G>, G: GenericNode> Clone for ElementBuilder<'a, E, G> {
    fn clone(&self) -> Self {
        Self {
            cx: self.cx,
            el: self.el.clone(),
            is_dyn: self.is_dyn,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, E: TypedElement<G>, G: GenericNode> AsNode<G> for ElementBuilder<'a, E, G> {
    fn as_node(&self) -> &G {
        self.el.as_node()
    }
}

impl<'a, E: TypedElement<G>, G: GenericNode> fmt::Debug for ElementBuilder<'a, E, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementBuilder")
            .field("element", self.el.as_node())
            .finish()
    }
}

/// A trait that allows either [`ElementBuilder`] or any type that implements
/// [`ToView`] to be used as a child.
pub trait ElementBuilderOrView<G: GenericNode, S> {
    /// If this is an [`ElementBuilder`], then it is converted to a [`View`]. Otherwise,
    /// [`ToView::to_view`] is called.
    fn into_view(self, cx: Scope) -> View<G>;
}
impl<'a, E: TypedElement<G>, G: GenericNode> ElementBuilderOrView<G, ()>
    for ElementBuilder<'a, E, G>
{
    fn into_view(self, _cx: Scope) -> View<G> {
        self.view()
    }
}
impl<G: GenericNode, T: ToView<G>> ElementBuilderOrView<G, ((),)> for T {
    fn into_view(self, cx: Scope) -> View<G> {
        self.to_view(cx)
    }
}

/// The `view!` spread operator.
///
/// This is most often used to spread a list of attributes onto an element.
pub trait Spread<'a, E: TypedElement<G>, G: GenericNode> {
    /// Spread onto the element.
    fn spread(self, cx: Scope<'a>, el: &G);
}

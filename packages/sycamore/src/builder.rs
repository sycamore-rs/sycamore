//! The builder pattern API for creating UI elements.
//!
//! This API is rendering-backend agnostic and can be used with any rendering backend, not just
//! HTML.

use js_sys::Reflect;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::component::component_scope;
use crate::generic_node::{GenericNode, Html, SycamoreElement};
use crate::noderef::NodeRef;
use crate::reactive::*;
use crate::utils::render;
use crate::view::View;

/// The prelude for the builder API. This is independent from the _sycamore prelude_, aka.
/// [`sycamore::prelude`].
///
/// In most cases, it is idiomatic to use a glob import (aka wildcard import) at the beginning of
/// your Rust source file.
///
/// ```rust
/// use sycamore::builder::prelude::*;
/// use sycamore::prelude::*;
/// ```
pub mod prelude {
    pub use super::component;
    pub use super::fragment;
    pub use super::h;
    pub use super::node;
    pub use crate::html::*;
}

/// A factory for building [`View`]s.
pub struct ElementBuilder<'a, G: GenericNode, F: FnOnce(ScopeRef<'a>) -> G + 'a>(
    F,
    PhantomData<&'a ()>,
);

/// A trait that is implemented only for [`ElementBuilder`] and [`View`].
/// This should be considered implementation details and should not be used.
pub trait ElementBuilderOrView<'a, G: GenericNode> {
    fn into_view(self, ctx: ScopeRef<'a>) -> View<G>;
}

impl<'a, G: GenericNode> ElementBuilderOrView<'a, G> for View<G> {
    fn into_view(self, _: ScopeRef<'a>) -> View<G> {
        self
    }
}

impl<'a, G: GenericNode, F: FnOnce(ScopeRef<'a>) -> G + 'a> ElementBuilderOrView<'a, G>
    for ElementBuilder<'a, G, F>
{
    fn into_view(self, ctx: ScopeRef<'a>) -> View<G> {
        self.view(ctx)
    }
}

/// Construct a new [`ElementBuilder`] from a [`SycamoreElement`].
///
/// # Example
/// TODO
pub fn h<'a, E: SycamoreElement, G: GenericNode>(
    _: E,
) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G> {
    ElementBuilder::new(move |_| G::element(E::TAG_NAME))
}

impl<'a, G: GenericNode, F: FnOnce(ScopeRef<'a>) -> G + 'a> ElementBuilder<'a, G, F> {
    fn new(f: F) -> Self {
        Self(f, PhantomData)
    }

    /// Utility function for composing new [`ElementBuilder`]s.
    fn map(
        self,
        f: impl FnOnce(ScopeRef<'a>, &G) + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        ElementBuilder::new(move |ctx| {
            let el = (self.0)(ctx);
            f(ctx, &el);
            el
        })
    }

    /// Set the attribute of the element.
    pub fn attr(
        self,
        name: &'a str,
        value: impl AsRef<str> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |_, el| el.set_attribute(name, value.as_ref()))
    }

    /// Set the boolean attribute of the element.
    pub fn bool_attr(
        self,
        name: &'a str,
        value: bool,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |_, el| {
            if value {
                el.set_attribute(name, "");
            }
        })
    }

    pub fn dyn_attr<S: AsRef<str> + 'a>(
        self,
        name: &'a str,
        mut value: impl FnMut() -> Option<S> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| {
            let el = el.clone();
            ctx.create_effect(move || {
                let value = value();
                if let Some(value) = value {
                    el.set_attribute(name, value.as_ref());
                } else {
                    el.remove_attribute(name);
                }
            });
        })
    }

    pub fn dyn_bool_attr(
        self,
        name: &'a str,
        mut value: impl FnMut() -> bool + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| {
            let el = el.clone();
            ctx.create_effect(move || {
                if value() {
                    el.set_attribute(name, "");
                } else {
                    el.remove_attribute(name);
                }
            });
        })
    }

    /// A shorthand for [`Self::attr`] for setting the class of the element.
    pub fn class(
        self,
        class: impl AsRef<str> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |_, el| el.add_class(class.as_ref()))
    }

    /// Adds a dynamic class on the node.
    ///
    /// If `value` is `None`, the class will be removed from the element.
    ///
    /// # Example
    /// TODO
    pub fn dyn_class(
        self,
        class: impl AsRef<str> + 'a,
        mut apply: impl FnMut() -> bool + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| {
            let el = el.clone();
            ctx.create_effect(move || {
                if apply() {
                    el.add_class(class.as_ref());
                } else {
                    el.remove_class(class.as_ref());
                }
            });
        })
    }

    /// A shorthand for [`Self::attr`] for setting the id of the element.
    pub fn id(
        self,
        class: impl AsRef<str> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |_, el| el.set_attribute("id", class.as_ref()))
    }

    /// Set a property on the element.
    pub fn prop(
        self,
        name: impl AsRef<str> + 'a,
        property: impl Into<JsValue> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |_, el| el.set_property(name.as_ref(), &property.into()))
    }

    /// Set a dynamic property on the element.
    pub fn dyn_prop<V: Into<JsValue> + 'a>(
        self,
        name: impl AsRef<str> + 'a,
        mut property: impl FnMut() -> V + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| {
            let el = el.clone();
            ctx.create_effect(move || {
                el.set_property(name.as_ref(), &property().into());
            });
        })
    }

    /// Insert a text node under this element. The inserted child is static by default.
    pub fn t(self, text: &'a str) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(|_, el| el.append_child(&G::text_node(text)))
    }

    pub fn dyn_t<S: AsRef<str> + 'a>(
        self,
        f: impl FnMut() -> S + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(|ctx, el| {
            let memo = ctx.create_memo(f);
            Self::dyn_c_internal(ctx, el, move || {
                View::new_node(G::text_node(memo.get().as_ref().as_ref()))
            });
        })
    }

    /// Insert a child node under this element. The inserted child is static by default.
    pub fn c(
        self,
        c: impl ElementBuilderOrView<'a, G>,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(|ctx, el| render::insert(ctx, el, c.into_view(ctx), None, None, true))
    }

    /// Internal implementation for [`Self::dyn_c`] and [`Self::dyn_t`].
    fn dyn_c_internal(ctx: ScopeRef<'a>, el: &G, f: impl FnMut() -> View<G> + 'a) {
        #[allow(unused_imports)]
        use std::any::{Any, TypeId};

        #[cfg(feature = "ssr")]
        if TypeId::of::<G>() == TypeId::of::<crate::generic_node::SsrNode>() {
            // If Server Side Rendering, insert beginning tag for hydration purposes.
            el.append_child(&G::marker_with_text("#"));
            // Create end marker. This is needed to make sure that the node is inserted into the
            // right place.
            let end_marker = G::marker_with_text("/");
            el.append_child(&end_marker);
            render::insert(
                ctx,
                el,
                View::new_dyn(ctx, f),
                None,
                Some(&end_marker),
                true, /* We don't know if this is the only child or not so we pessimistically
                       * set this to true. */
            );
            return;
        }
        #[cfg(feature = "experimental-hydrate")]
        if TypeId::of::<G>() == TypeId::of::<crate::generic_node::HydrateNode>() {
            use crate::utils::hydrate::web::*;
            // Get start and end markers.
            let el_hn = <dyn Any>::downcast_ref::<crate::generic_node::HydrateNode>(el).unwrap();
            let initial = get_next_marker(&el_hn.inner_element());
            // Do not drop the HydrateNode because it will be cast into a GenericNode.
            let initial = ::std::mem::ManuallyDrop::new(initial);
            // SAFETY: This is safe because we already checked that the type is HydrateNode.
            // __initial is wrapped inside ManuallyDrop to prevent double drop.
            let initial = unsafe { ::std::ptr::read(&initial as *const _ as *const _) };
            render::insert(
                ctx,
                el,
                View::new_dyn(ctx, f),
                initial,
                None,
                true, /* We don't know if this is the only child or not so we pessimistically
                       * set this to true. */
            );
            return;
        }
        // G is neither SsrNode nor HydrateNode. Proceed normally.
        let marker = G::marker();
        el.append_child(&marker);
        render::insert(ctx, el, View::new_dyn(ctx, f), None, Some(&marker), true);
    }

    pub fn dyn_c<O: ElementBuilderOrView<'a, G> + 'a>(
        self,
        mut f: impl FnMut() -> O + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| Self::dyn_c_internal(ctx, el, move || f().into_view(ctx)))
    }

    pub fn dyn_if<O1: ElementBuilderOrView<'a, G> + 'a, O2: ElementBuilderOrView<'a, G> + 'a>(
        self,
        cond: impl Fn() -> bool + 'a,
        mut then: impl FnMut() -> O1 + 'a,
        mut r#else: impl FnMut() -> O2 + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        let cond = Rc::new(cond);
        self.map(move |ctx, el| {
            Self::dyn_c_internal(ctx, el, move || {
                if *ctx
                    .create_selector({
                        let cond = Rc::clone(&cond);
                        #[allow(clippy::redundant_closure)] // FIXME: clippy false positive
                        move || cond()
                    })
                    .get()
                {
                    then().into_view(ctx)
                } else {
                    r#else().into_view(ctx)
                }
            });
        })
    }

    pub fn dyn_c_scoped(
        self,
        f: impl FnMut(BoundedScopeRef<'_, 'a>) -> View<G> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| {
            #[allow(unused_imports)]
            use std::any::{Any, TypeId};

            #[cfg(feature = "ssr")]
            if TypeId::of::<G>() == TypeId::of::<crate::generic_node::SsrNode>() {
                // If Server Side Rendering, insert beginning tag for hydration purposes.
                el.append_child(&G::marker_with_text("#"));
                // Create end marker. This is needed to make sure that the node is inserted into the
                // right place.
                let end_marker = G::marker_with_text("/");
                el.append_child(&end_marker);
                render::insert(
                    ctx,
                    el,
                    View::new_dyn_scoped(ctx, f),
                    None,
                    Some(&end_marker),
                    true, /* We don't know if this is the only child or not so we
                           * pessimistically set this to true. */
                );
                return;
            }
            #[cfg(feature = "experimental-hydrate")]
            if TypeId::of::<G>() == TypeId::of::<crate::generic_node::HydrateNode>() {
                use crate::utils::hydrate::web::*;
                // Get start and end markers.
                let el_hn =
                    <dyn Any>::downcast_ref::<crate::generic_node::HydrateNode>(el).unwrap();
                let initial = get_next_marker(&el_hn.inner_element());
                // Do not drop the HydrateNode because it will be cast into a GenericNode.
                let initial = ::std::mem::ManuallyDrop::new(initial);
                // SAFETY: This is safe because we already checked that the type is HydrateNode.
                // __initial is wrapped inside ManuallyDrop to prevent double drop.
                let initial = unsafe { ::std::ptr::read(&initial as *const _ as *const _) };
                render::insert(
                    ctx,
                    el,
                    View::new_dyn_scoped(ctx, f),
                    initial,
                    None,
                    true, /* We don't know if this is the only child or not so we
                           * pessimistically set this to true. */
                );
                return;
            }
            // G is neither SsrNode nor HydrateNode. Proceed normally.
            let marker = G::marker();
            el.append_child(&marker);
            render::insert(
                ctx,
                el,
                View::new_dyn_scoped(ctx, f),
                None,
                Some(&marker),
                true,
            );
        })
    }

    /// Attach an event handler to the element.
    pub fn on(
        self,
        name: &'a str,
        handler: impl Fn(G::EventType) + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| el.event(ctx, name, Box::new(handler)))
    }

    /// Get a hold of the raw element by using a [`NodeRef`].
    pub fn bind_ref(
        self,
        node_ref: NodeRef<G>,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |_, el| node_ref.set(el.clone()))
    }

    /// Construct a [`View`] by evaluating the lazy [`ElementBuilder`].
    pub fn view(self, ctx: ScopeRef<'a>) -> View<G> {
        let el = (self.0)(ctx);
        View::new_node(el)
    }
}

/// HTML-specific builder methods.
impl<'a, G: Html, F: FnOnce(ScopeRef<'a>) -> G + 'a> ElementBuilder<'a, G, F> {
    /// Binds a [`Signal`] to the `value` property of the node.
    ///
    /// The [`Signal`] will be automatically updated when the value is updated.
    ///
    /// # Example
    /// TODO
    pub fn bind_value(
        self,
        sub: &'a Signal<String>,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| {
            ctx.create_effect({
                let el = el.clone();
                move || {
                    el.set_property("value", &sub.get().as_str().into());
                }
            });
            el.event(
                ctx,
                "input",
                Box::new(move |e: web_sys::Event| {
                    let val = Reflect::get(
                        &e.target().expect("missing target on input event"),
                        &"value".into(),
                    )
                    .expect("missing property `value`")
                    .as_string()
                    .expect("value should be a string");
                    sub.set(val);
                }),
            );
        })
    }

    /// Binds a [`Signal`] to the `checked` property of the node.
    ///
    /// The [`Signal`] will be automatically updated when the value is updated.
    ///
    /// # Example
    /// TODO
    pub fn bind_checked(
        self,
        sub: &'a Signal<bool>,
    ) -> ElementBuilder<'a, G, impl FnOnce(ScopeRef<'a>) -> G + 'a> {
        self.map(move |ctx, el| {
            ctx.create_effect({
                let el = el.clone();
                move || {
                    el.set_property("checked", &(*sub.get()).into());
                }
            });
            el.event(
                ctx,
                "change",
                Box::new(move |e: web_sys::Event| {
                    let val = Reflect::get(
                        &e.target().expect("missing target on change event"),
                        &"checked".into(),
                    )
                    .expect("missing property `checked`")
                    .as_bool()
                    .expect("could not get property `checked` as a bool");
                    sub.set(val);
                }),
            );
        })
    }
}

pub fn view<'a, G: GenericNode>(ctx: ScopeRef<'a>, f: impl FnOnce(ScopeRef<'a>) -> G) -> View<G> {
    View::new_node(f(ctx))
}

/// Create [`NodeBuilder`] to create UI elements.
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # fn _test<G: GenericNode>() -> View<G> {
/// node("div").build()
/// # }
/// # fn _test2<G: GenericNode>() -> View<G> {
/// node("a").build()
/// # }
/// ```
pub fn node<'a, G>(ctx: ScopeRef<'a>, tag: &'static str) -> NodeBuilder<'a, G>
where
    G: GenericNode,
{
    NodeBuilder {
        ctx,
        element: G::element(tag),
    }
}

/// Instantiate a component as a [`View`].
///
/// # Example
/// ```
/// use sycamore::prelude::*;
/// # use sycamore::builder::html::*;
/// #[component(MyComponent<G>)]
/// fn my_component() -> View<G> {
///     h1().text("I am a component").build()
/// }
///
/// // Elsewhere in another component.
/// # fn view<G: Html>() -> View<G> {
/// component::<_, MyComponent<_>>(())
/// # }
/// ```
pub fn component<G>(f: impl FnOnce() -> View<G>) -> View<G>
where
    G: GenericNode + Html,
{
    component_scope(f)
}

/// Create a [`View`] from an array of [`View`].
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # use sycamore::builder::html::*;
/// # fn _test<G: GenericNode>() -> View<G> {
/// fragment([
///     div().build(),
///     div().build()
/// ])
/// # }
/// ```
pub fn fragment<G, const N: usize>(parts: [View<G>; N]) -> View<G>
where
    G: GenericNode,
{
    View::new_fragment(Vec::from_iter(parts.to_vec()))
}

/// The main type powering the builder API.
#[derive(Clone)]
pub struct NodeBuilder<'a, G>
where
    G: GenericNode,
{
    ctx: ScopeRef<'a>,
    element: G,
}

impl<'a, G> NodeBuilder<'a, G>
where
    G: GenericNode,
{
    /// Add a child [`View`].
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    ///  div()
    ///     .child(h1().text("I am a child").build())
    ///     .build()
    /// # }
    /// ```
    pub fn child(&self, child: View<G>) -> &Self {
        render::insert(self.ctx, &self.element, child, None, None, true);

        self
    }

    /// Add a dynamic child [`View`]
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// let visible = Signal::new(true);
    ///
    /// div()
    ///     .dyn_child(
    ///         move || {
    ///             if *visible.get() {
    ///                 h1().text("I am a child").build()
    ///             } else { View::empty() }
    ///         }
    ///     )
    ///     .build()
    /// # }
    /// ```
    pub fn dyn_child(&self, child: impl FnMut() -> View<G> + 'a) -> &Self {
        #[allow(unused_imports)]
        use std::any::{Any, TypeId};

        #[cfg(feature = "ssr")]
        if TypeId::of::<G>() == TypeId::of::<crate::generic_node::SsrNode>() {
            // If Server Side Rendering, insert beginning tag for hydration purposes.
            self.element.append_child(&G::marker_with_text("#"));
            // Create end marker. This is needed to make sure that the node is inserted into the
            // right place.
            let end_marker = G::marker_with_text("/");
            self.element.append_child(&end_marker);
            render::insert(
                self.ctx,
                &self.element,
                View::new_dyn(self.ctx, child),
                None,
                Some(&end_marker),
                true, /* We don't know if this is the only child or not so we pessimistically
                       * set this to true. */
            );
            return self;
        }
        #[cfg(feature = "experimental-hydrate")]
        if TypeId::of::<G>() == TypeId::of::<crate::generic_node::HydrateNode>() {
            use crate::utils::hydrate::web::*;
            // Get start and end markers.
            let el =
                <dyn Any>::downcast_ref::<crate::generic_node::HydrateNode>(&self.element).unwrap();
            let initial = get_next_marker(&el.inner_element());
            // Do not drop the HydrateNode because it will be cast into a GenericNode.
            let initial = ::std::mem::ManuallyDrop::new(initial);
            // SAFETY: This is safe because we already checked that the type is HydrateNode.
            // __initial is wrapped inside ManuallyDrop to prevent double drop.
            let initial = unsafe { ::std::ptr::read(&initial as *const _ as *const _) };
            render::insert(
                self.ctx,
                &self.element,
                View::new_dyn(self.ctx, child),
                initial,
                None,
                true, /* We don't know if this is the only child or not so we pessimistically
                       * set this to true. */
            );
            return self;
        }
        // G is neither SsrNode nor HydrateNode. Proceed normally.
        let marker = G::marker();
        self.element.append_child(&marker);
        render::insert(
            self.ctx,
            &self.element,
            View::new_dyn(self.ctx, child),
            None,
            Some(&marker),
            true,
        );
        self
    }

    /// Adds a text node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// h1().text("I am text").build()
    /// }
    /// ```
    pub fn text(&self, text: impl AsRef<str>) -> &Self {
        self.element.append_child(&G::text_node(text.as_ref()));

        self
    }

    /// Adds a dynamic text node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// let required = Signal::new(false);
    ///
    /// h1()
    ///     .text("Email")
    ///     .dyn_text(
    ///         move || {
    ///             if *required.get() { " *" } else { "" }
    ///         }
    ///     ).build()
    /// }
    /// ```
    pub fn dyn_text<F, O>(&self, text: F) -> &Self
    where
        F: FnMut() -> O + 'a,
        O: AsRef<str> + 'static,
    {
        let memo = self.ctx.create_memo(text);

        self.dyn_child(move || View::new_node(G::text_node(memo.get().as_ref().as_ref())));

        self
    }

    /// Renders a component as a child.
    ///
    /// # Example
    /// ```
    /// use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// #[component(MyComponent<G>)]
    /// fn my_component() -> View<G> {
    ///     h1().text("My component").build()
    /// }
    ///
    /// # fn _test<G: Html>() -> View<G> {
    /// div().component::<MyComponent<_>>(()).build()
    /// }
    /// ```
    pub fn component(&self, f: impl FnOnce() -> View<G>) -> &Self {
        self.child(component_scope(f));

        self
    }

    /// Convenience function for adding an `id` to a node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// button().id("my-button").build()
    /// # }
    /// ```
    pub fn id(&self, id: impl AsRef<str>) -> &Self {
        self.attr("id", id.as_ref())
    }

    /// Set an attribute on the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// button().attr("type", "submit").build()
    /// # }
    /// ```
    pub fn attr<N, Va>(&self, name: N, value: Va) -> &Self
    where
        N: AsRef<str>,
        Va: AsRef<str>,
    {
        self.element.set_attribute(name.as_ref(), value.as_ref());

        self
    }

    /// Set a boolean attribute on the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// input().bool_attr("required", true).build()
    /// # }
    /// ```
    pub fn bool_attr<N>(&self, name: N, value: bool) -> &Self
    where
        N: AsRef<str>,
    {
        if value {
            self.attr(name.as_ref(), "");
        } else {
            self.element.remove_attribute(name.as_ref());
        }

        self
    }

    /// Adds a dynamic attribute on the node.
    ///
    /// If `value` is `None`, the attribute will be completely removed
    /// from the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// let input_type = Signal::new(Some("text"));
    ///
    /// input()
    ///     .dyn_attr("type", input_type.handle())
    ///     .build()
    /// }
    /// ```
    pub fn dyn_attr<N, T>(&self, name: N, value: ReadSignal<Option<T>>) -> &Self
    where
        N: ToString,
        T: ToString + 'a,
    {
        let element = self.element.clone();

        let name = name.to_string();

        self.ctx.create_effect(move || {
            let v = value.get();

            if let Some(v) = &*v {
                element.set_attribute(name.as_ref(), v.to_string().as_ref());
            } else {
                element.remove_attribute(name.as_ref());
            }
        });

        self
    }

    /// Adds a dynamic boolean attribute on the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// let required = Signal::new(true);
    ///
    /// input()
    ///     .dyn_bool_attr("required", required.handle()).build()
    /// }
    /// ```
    pub fn dyn_bool_attr<N>(&self, name: N, value: ReadSignal<bool>) -> &Self
    where
        N: ToString,
    {
        let element = self.element.clone();

        let name = name.to_string();

        self.ctx.create_effect(move || {
            let v = value.get();

            if *v {
                element.set_attribute(name.as_ref(), "");
            } else {
                element.remove_attribute(name.as_ref());
            }
        });

        self
    }

    /// Set a property on the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// input().prop("value", "I am the value set.").build()
    /// # }
    /// ```
    pub fn prop<N, Va>(&self, name: N, property: Va) -> &Self
    where
        N: AsRef<str>,
        Va: Into<JsValue>,
    {
        self.element.set_property(name.as_ref(), &property.into());

        self
    }

    /// Adds a dynamic property on the node.
    ///
    /// If `value` is `None`, the attribute will be completely removed
    /// from the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// let checked = Signal::new(Some(false));
    ///
    /// input()
    ///     .attr("type", "checkbox")
    ///     .dyn_prop("checked", checked.handle())
    ///     .build()
    /// }
    /// ```
    pub fn dyn_prop<N, T>(&self, name: N, value: &'a ReadSignal<Option<T>>) -> &Self
    where
        N: ToString,
        T: ToString + 'a,
    {
        let element = self.element.clone();

        let name = name.to_string();

        self.ctx.create_effect(move || {
            let v = value.get();

            if let Some(v) = &*v {
                element.set_attribute(name.as_ref(), v.to_string().as_ref());
            } else {
                element.remove_attribute(name.as_ref());
            }
        });

        self
    }

    /// Adds a class to the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// button().class("bg-green-500").text("My Button").build()
    /// # }
    /// ```
    pub fn class(&self, class: impl ToString) -> &Self {
        self.element.add_class(class.to_string().as_ref());

        self
    }

    /// Adds a dynamic class on the node.
    ///
    /// If `value` is `None`, the attribute will be completely removed
    /// from the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// let checked_class = Signal::new(false);
    ///
    /// input()
    ///     .attr("type", "checkbox")
    ///     .dyn_class("bg-red-500", checked_class.handle())
    ///     .build()
    /// }
    /// ```
    pub fn dyn_class(&self, class: impl ToString, apply: ReadSignal<bool>) -> &Self {
        let class = class.to_string();
        let element = self.element.clone();

        self.ctx.create_effect(move || {
            let apply = apply.get();

            if *apply {
                element.add_class(class.as_ref());
            } else {
                element.remove_class(class.as_ref());
            }
        });

        self
    }

    #[allow(dead_code)]
    #[doc(hidden)]
    fn styles(&self, styles: HashMap<String, String>) -> &Self {
        let styles = styles
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(";");

        self.attr("style", styles);

        self
    }

    /// Adds an event listener to the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// button()
    ///     .text("My Button")
    ///     .on(
    ///         "click",
    ///         |_| { web_sys::console::log_1(&"Clicked".into()) }
    ///     )
    ///     .build()
    /// # }
    /// ```
    pub fn on<E, H>(&self, event: E, handler: H) -> &Self
    where
        E: AsRef<str>,
        H: Fn(G::EventType) + 'a,
    {
        self.element
            .event(self.ctx, event.as_ref(), Box::new(handler));

        self
    }

    /// Get a hold of the raw element by using a [`NodeRef`].
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// let node_ref = NodeRef::new();
    ///
    /// input()
    ///     .bind_ref(node_ref.clone())
    ///     .build()
    /// # }
    /// ```
    pub fn bind_ref(&self, node_ref: NodeRef<G>) -> &Self {
        node_ref.set(self.element.clone());

        self
    }

    /// Builds the [`NodeBuilder`] and returns a [`View`].
    ///
    /// This is the function that should be called at the end of the node
    /// building chain.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> View<G> {
    /// input()
    ///     /* builder stuff */
    ///     .build()
    /// # }
    /// ```
    pub fn build(&self) -> View<G> {
        View::new_node(self.element.to_owned())
    }
}

impl<'a, G> NodeBuilder<'a, G>
where
    G: GenericNode + Html,
{
    /// Binds `sub` to the `value` property of the node.
    ///
    /// `sub` will be automatically updated when the value is updated.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// let value = Signal::new(String::new());
    ///
    /// input()
    ///     .bind_value(value)
    ///     .build()
    /// # }
    /// ```
    pub fn bind_value(&self, sub: &'a Signal<String>) -> &Self {
        let sub_handle = self.ctx.create_memo(|| Some((*sub.get()).clone()));

        self.dyn_prop("value", sub_handle);

        self.on("input", move |e| {
            let value = Reflect::get(
                &e.target()
                    .expect("Target missing on input event.")
                    .unchecked_into::<web_sys::Element>(),
                &"value".into(),
            )
            .expect("Missing value prop.")
            .as_string()
            .expect("Value should be a string.");

            sub.set(value);
        })
    }

    /// Binds `sub` to the `checked` property of the node.
    ///
    /// `sub` will be automatically updated when the value is updated.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// let checked = Signal::new(false);
    ///
    /// input()
    ///     .attr("type", "checkbox")
    ///     .bind_checked(checked)
    ///     .build()
    /// # }
    /// ```
    pub fn bind_checked(&self, sub: &'a Signal<bool>) -> &Self {
        let sub_handle = self.ctx.create_memo(|| Some(*sub.get()));

        self.dyn_prop("checked", sub_handle);

        self.on("change", move |e| {
            let value = Reflect::get(
                &e.target().expect("Target missing on change event."),
                &"checked".into(),
            )
            .expect("Failed to get checked prop.")
            .as_bool();

            if let Some(value) = value {
                sub.set(value);
            } else {
                panic!(
                    "Checked is only available on input elements with type attribute set to checkbox."
                );
            }
        })
    }
}

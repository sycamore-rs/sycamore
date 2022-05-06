//! The builder pattern API for creating UI elements.
//!
//! This API is rendering-backend agnostic and can be used with any rendering backend, not just
//! HTML.

use std::iter::FromIterator;
use std::marker::PhantomData;
use std::rc::Rc;

use js_sys::Reflect;

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
    pub use super::{component, dyn_t, fragment, h, t, tag};
    pub use crate::html::*;
}

/// A factory for building [`View`]s.
pub struct ElementBuilder<'a, G: GenericNode, F: FnOnce(Scope<'a>) -> G + 'a>(
    F,
    PhantomData<&'a ()>,
);

/// A trait that is implemented only for [`ElementBuilder`] and [`View`].
/// This should be considered implementation details and should not be used.
pub trait ElementBuilderOrView<'a, G: GenericNode> {
    /// Convert into a [`View`].
    fn into_view(self, cx: Scope<'a>) -> View<G>;
}

impl<'a, G: GenericNode> ElementBuilderOrView<'a, G> for View<G> {
    fn into_view(self, _: Scope<'a>) -> View<G> {
        self
    }
}

impl<'a, G: GenericNode, F: FnOnce(Scope<'a>) -> G + 'a> ElementBuilderOrView<'a, G>
    for ElementBuilder<'a, G, F>
{
    fn into_view(self, cx: Scope<'a>) -> View<G> {
        self.view(cx)
    }
}

/// Construct a new [`ElementBuilder`] from a [`SycamoreElement`].
///
/// Note that this can not be used to construct custom elements because they are not type checked in
/// Rust. You'll need to use the [`tag`] function instead.
///
/// # Example
/// ```
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// # fn _test1<G: GenericNode>(cx: Scope) -> View<G> {
/// h(a)
/// # .view(cx) }
/// # fn _test2<G: GenericNode>(cx: Scope) -> View<G> {
/// h(button)
/// # .view(cx) }
/// # fn _test3<G: GenericNode>(cx: Scope) -> View<G> {
/// h(div)
/// # .view(cx) }
/// // etc...
/// ```
pub fn h<'a, E: SycamoreElement, G: GenericNode>(
    _: E,
) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G> {
    ElementBuilder::new(move |_| G::element::<E>())
}

/// Construct a new [`ElementBuilder`] from a tag name.
/// Generally, it is preferable to use [`h`] instead unless using custom elements.
///
/// # Example
/// ```
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// # fn _test1<G: GenericNode>(cx: Scope) -> View<G> {
/// tag("a")
/// # .view(cx) }
/// # fn _test2<G: GenericNode>(cx: Scope) -> View<G> {
/// tag("button")
/// # .view(cx) }
/// # fn _test3<G: GenericNode>(cx: Scope) -> View<G> {
/// tag("my-custom-element")
/// # .view(cx) }
/// // etc...
/// ```
pub fn tag<'a, G: GenericNode>(
    t: impl AsRef<str>,
) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G> {
    ElementBuilder::new(move |_| G::element_from_tag(t.as_ref()))
}

impl<'a, G: GenericNode, F: FnOnce(Scope<'a>) -> G + 'a> ElementBuilder<'a, G, F> {
    fn new(f: F) -> Self {
        Self(f, PhantomData)
    }

    /// Utility function for composing new [`ElementBuilder`]s.
    fn map(
        self,
        f: impl FnOnce(Scope<'a>, &G) + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        ElementBuilder::new(move |cx| {
            let el = (self.0)(cx);
            f(cx, &el);
            el
        })
    }

    /// Set the attribute of the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// h(button).attr("type", "submit")
    /// # .view(cx) }
    /// ```
    pub fn attr(
        self,
        name: &'a str,
        value: impl AsRef<str> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |_, el| el.set_attribute(name, value.as_ref()))
    }

    /// Set the boolean attribute of the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// h(input).bool_attr("required", true)
    /// # .view(cx) }
    /// ```
    pub fn bool_attr(
        self,
        name: &'a str,
        value: bool,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |_, el| {
            if value {
                el.set_attribute(name, "");
            }
        })
    }

    /// Adds a dynamic attribute on the node.
    ///
    /// If `value` is `None`, the attribute will be removed from the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let input_type = create_signal(cx, "text");
    /// h(input).dyn_attr("type", || Some(*input_type.get()))
    /// # .view(cx) }
    /// ```
    pub fn dyn_attr<S: AsRef<str> + 'a>(
        self,
        name: &'a str,
        mut value: impl FnMut() -> Option<S> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |cx, el| {
            let el = el.clone();
            create_effect(cx, move || {
                let value = value();
                if let Some(value) = value {
                    el.set_attribute(name, value.as_ref());
                } else {
                    el.remove_attribute(name);
                }
            });
        })
    }

    /// Adds a dynamic boolean attribute on the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let required = create_signal(cx, true);
    /// h(input).dyn_bool_attr("required", || *required.get())
    /// # .view(cx) }
    /// ```
    pub fn dyn_bool_attr(
        self,
        name: &'a str,
        mut value: impl FnMut() -> bool + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |cx, el| {
            let el = el.clone();
            create_effect(cx, move || {
                if value() {
                    el.set_attribute(name, "");
                } else {
                    el.remove_attribute(name);
                }
            });
        })
    }

    /// Adds a class to the element. This is a shorthand for [`Self::attr`] with the `class`
    /// attribute.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// h(button).class("bg-green-500").t("My button")
    /// # .view(cx) }
    /// ```
    pub fn class(
        self,
        class: impl AsRef<str> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |_, el| el.add_class(class.as_ref()))
    }

    /// Adds a dynamic class on the node.
    ///
    /// If `value` is `None`, the class will be removed from the element.
    ///
    /// # Example
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let checked_class = create_signal(cx, false);
    /// h(input)
    ///     .attr("type", "checkbox")
    ///     .dyn_class("bg-red-500", || *checked_class.get())
    /// # .view(cx) }
    /// ```
    pub fn dyn_class(
        self,
        class: impl AsRef<str> + 'a,
        mut apply: impl FnMut() -> bool + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |cx, el| {
            let el = el.clone();
            create_effect(cx, move || {
                if apply() {
                    el.add_class(class.as_ref());
                } else {
                    el.remove_class(class.as_ref());
                }
            });
        })
    }

    /// Sets the id of an element. This is a shorthand for [`Self::attr`] with the `id` attribute.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// h(button).id("my-button")
    /// # .view(cx) }
    /// ```
    pub fn id(
        self,
        class: impl AsRef<str> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |_, el| el.set_attribute("id", class.as_ref()))
    }

    /// Set a property on the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// h(input).prop("value", "I am the value set.")
    /// # .view(cx) }
    /// ```
    pub fn prop(
        self,
        name: impl AsRef<str> + 'a,
        property: impl Into<G::PropertyType> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |_, el| el.set_property(name.as_ref(), &property.into()))
    }

    /// Set a dynamic property on the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let checked = create_signal(cx, false);
    /// h(input)
    ///     .attr("type", "checkbox")
    ///     .dyn_prop("checked", || *checked.get())
    /// # .view(cx) }
    /// ```
    pub fn dyn_prop<V: Into<G::PropertyType> + 'a>(
        self,
        name: impl AsRef<str> + 'a,
        mut property: impl FnMut() -> V + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |cx, el| {
            let el = el.clone();
            create_effect(cx, move || {
                el.set_property(name.as_ref(), &property().into());
            });
        })
    }

    /// Insert a text node under this element. The inserted child is static by default.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// h(p)
    ///     .t("Hello World!")
    ///     .t("Text nodes can be chained as well.")
    ///     .t("More text...")
    /// # .view(cx) }
    /// ```
    pub fn t(self, text: &'a str) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(|_, el| el.append_child(&G::text_node(text)))
    }

    /// Adds a dynamic text node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let name = create_signal(cx, "Sycamore");
    /// h(p)
    ///     .t("Name: ")
    ///     .dyn_t(|| name.get().to_string())
    /// # .view(cx) }
    /// ```
    pub fn dyn_t<S: AsRef<str> + 'a>(
        self,
        f: impl FnMut() -> S + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(|cx, el| {
            let memo = create_memo(cx, f);
            Self::dyn_c_internal(cx, el, move || {
                View::new_node(G::text_node(memo.get().as_ref().as_ref()))
            });
        })
    }

    /// Insert a child node under this element. The inserted child is static by default.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let input_type = create_signal(cx, "text");
    /// h(div).c(
    ///     h(h1).t("I am a child")
    /// )
    /// # .view(cx) }
    /// ```
    pub fn c(
        self,
        c: impl ElementBuilderOrView<'a, G>,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(|cx, el| render::insert(cx, el, c.into_view(cx), None, None, true))
    }

    /// Internal implementation for [`Self::dyn_c`] and [`Self::dyn_t`].
    fn dyn_c_internal(cx: Scope<'a>, el: &G, f: impl FnMut() -> View<G> + 'a) {
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
                cx,
                el,
                View::new_dyn(cx, f),
                None,
                Some(&end_marker),
                true, /* We don't know if this is the only child or not so we pessimistically
                       * set this to true. */
            );
            return;
        }
        #[cfg(feature = "hydrate")]
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
                cx,
                el,
                View::new_dyn(cx, f),
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
        render::insert(cx, el, View::new_dyn(cx, f), None, Some(&marker), true);
    }

    /// Internal implementation for [`Self::dyn_c_scoped`] and [`Self::dyn_if`].
    fn dyn_c_internal_scoped(
        cx: Scope<'a>,
        el: &G,
        f: impl FnMut(BoundedScope<'_, 'a>) -> View<G> + 'a,
    ) {
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
                cx,
                el,
                View::new_dyn_scoped(cx, f),
                None,
                Some(&end_marker),
                true, /* We don't know if this is the only child or not so we
                       * pessimistically set this to true. */
            );
            return;
        }
        #[cfg(feature = "hydrate")]
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
                cx,
                el,
                View::new_dyn_scoped(cx, f),
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
            cx,
            el,
            View::new_dyn_scoped(cx, f),
            None,
            Some(&marker),
            true,
        );
    }

    /// Adds a dynamic child. Note that most times, [`dyn_if`](Self::dyn_if) can be used instead
    /// which is more ergonomic.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn some_view<G: GenericNode>() -> View<G> { todo!() }
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let a_view = || some_view();
    /// h(div).dyn_c(a_view)
    /// # .view(cx) }
    /// ```
    pub fn dyn_c<O: ElementBuilderOrView<'a, G> + 'a>(
        self,
        mut f: impl FnMut() -> O + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |cx, el| Self::dyn_c_internal(cx, el, move || f().into_view(cx)))
    }

    /// Adds a dynamic, conditional view.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let visible = create_signal(cx, true);
    /// h(div).dyn_if(
    ///     || *visible.get(),
    ///     || h(p).t("Now you see me"),
    ///     || h(p).t("Now you don't!"),
    /// )
    /// # .view(cx) }
    /// ```
    pub fn dyn_if<O1: ElementBuilderOrView<'a, G> + 'a, O2: ElementBuilderOrView<'a, G> + 'a>(
        self,
        cond: impl Fn() -> bool + 'a,
        mut then: impl FnMut() -> O1 + 'a,
        mut r#else: impl FnMut() -> O2 + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        let cond = Rc::new(cond);
        self.map(move |cx, el| {
            // FIXME: should be dyn_c_internal_scoped to prevent memory leaks.
            Self::dyn_c_internal(cx, el, move || {
                if *create_selector(cx, {
                    let cond = Rc::clone(&cond);
                    #[allow(clippy::redundant_closure)] // FIXME: clippy false positive
                    move || cond()
                })
                .get()
                {
                    then().into_view(cx)
                } else {
                    r#else().into_view(cx)
                }
            });
        })
    }

    /// Adds a dynamic child that is created in a new reactive scope.
    ///
    /// [`dyn_c`](Self::dyn_c) uses [`create_effect`] whereas this method uses
    /// [`create_effect_scoped`].
    pub fn dyn_c_scoped(
        self,
        f: impl FnMut(BoundedScope<'_, 'a>) -> View<G> + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(|cx, el| Self::dyn_c_internal_scoped(cx, el, f))
    }

    /// Attach an event handler to the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// h(button)
    ///     .t("My button")
    ///     .on("click", |_| web_sys::console::log_1(&"Clicked".into()))
    /// # .view(cx) }
    /// ```
    pub fn on(
        self,
        name: &'a str,
        handler: impl Fn(G::EventType) + 'a,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |cx, el| el.event(cx, name, Box::new(handler)))
    }

    /// Get a hold of the raw element by using a [`NodeRef`].
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
    /// let node_ref = create_node_ref(cx);
    /// h(input).bind_ref(node_ref.clone())
    /// # .view(cx) }
    /// ```
    pub fn bind_ref(
        self,
        node_ref: NodeRef<G>,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |_, el| node_ref.set(el.clone()))
    }

    /// Construct a [`View`] by evaluating the lazy [`ElementBuilder`].
    ///
    /// This is the method that should be called at the end of the building chain.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// #[component]
    /// fn MyComponent<G: GenericNode>(cx: Scope) -> View<G> {
    ///     h(div)
    ///         /* builder stuff... */
    ///         .view(cx)
    /// }
    /// ```
    pub fn view(self, cx: Scope<'a>) -> View<G> {
        let el = (self.0)(cx);
        View::new_node(el)
    }
}

/// HTML-specific builder methods.
impl<'a, G: Html, F: FnOnce(Scope<'a>) -> G + 'a> ElementBuilder<'a, G, F> {
    /// Binds a [`Signal`] to the `value` property of the node.
    ///
    /// The [`Signal`] will be automatically updated when the value is updated.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>(cx: Scope) -> View<G> {
    /// let value = create_signal(cx, String::new());
    /// h(input).bind_value(value)
    /// # .view(cx) }
    /// ```
    pub fn bind_value(
        self,
        sub: &'a Signal<String>,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |cx, el| {
            create_effect(cx, {
                let el = el.clone();
                move || {
                    el.set_property("value", &sub.get().as_str().into());
                }
            });
            el.event(
                cx,
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
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>(cx: Scope) -> View<G> {
    /// let checked = create_signal(cx, true);
    /// h(input).attr("type", "checkbox").bind_checked(checked)
    /// # .view(cx) }
    /// ```
    pub fn bind_checked(
        self,
        sub: &'a Signal<bool>,
    ) -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G + 'a> {
        self.map(move |cx, el| {
            create_effect(cx, {
                let el = el.clone();
                move || {
                    el.set_property("checked", &(*sub.get()).into());
                }
            });
            el.event(
                cx,
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

/// Instantiate a component as a [`View`].
///
/// # Example
/// ```
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// #[component]
/// fn MyComponent<G: GenericNode>(cx: Scope) -> View<G> {
///     h(h1).t("I am a component").view(cx)
/// }
///
/// // Elsewhere...
/// # fn view<G: Html>(cx: Scope) -> View<G> {
/// component(|| MyComponent(cx, ()))
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
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
/// fragment([
///     h(div).view(cx),
///     h(div).view(cx),
/// ])
/// # }
/// ```
pub fn fragment<G, const N: usize>(parts: [View<G>; N]) -> View<G>
where
    G: GenericNode,
{
    View::new_fragment(Vec::from_iter(parts.to_vec()))
}

/// Construct a new top-level text [`View`].
///
/// # Example
/// ```
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// # fn _test1<G: GenericNode>(cx: Scope) -> View<G> {
/// t("Hello!")
/// # }
/// # fn _test2<G: GenericNode>(cx: Scope) -> View<G> {
/// t("This is top level text.")
/// # }
/// # fn _test3<G: GenericNode>(cx: Scope) -> View<G> {
/// t("We aren't directly nested under an element.")
/// # }
/// // etc...
/// ```
pub fn t<G: GenericNode>(t: impl AsRef<str>) -> View<G> {
    View::new_node(G::text_node(t.as_ref()))
}

/// Construct a new top-level dynamic text [`View`].
///
/// # Example
/// ```
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// # fn _test<G: GenericNode>(cx: Scope) -> View<G> {
/// dyn_t(cx, || "Hello!")
/// # }
/// ```
pub fn dyn_t<'a, G: GenericNode, S: AsRef<str>>(
    cx: Scope<'a>,
    mut f: impl FnMut() -> S + 'a,
) -> View<G> {
    View::new_dyn(cx, move || View::new_node(G::text_node(f().as_ref())))
}

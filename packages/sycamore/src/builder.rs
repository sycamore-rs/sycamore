//! The builder pattern API for creating UI elements.
//!
//! This API is rendering-backend agnostic and can be used with any rendering backend, not just
//! HTML.

use std::borrow::Cow;
use std::iter::FromIterator;
use std::rc::Rc;

use sycamore_core::event::{EventDescriptor, EventHandler};
use sycamore_core::generic_node::GenericNodeElements;

use crate::component::component_scope;
use crate::generic_node::GenericNode;
use crate::noderef::NodeRef;
use crate::reactive::*;
use crate::utils::render;
use crate::view::View;
use crate::web::html::ev;
#[cfg(feature = "web")]
use crate::web::Html;

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
    pub use super::{component, dyn_t, fragment, t, tag};
    #[cfg(feature = "web")]
    pub use crate::web::html::ev;
    #[cfg(feature = "web")]
    pub use crate::web::html::html_tags::builder::*;
    #[cfg(feature = "web")]
    pub use crate::web::html::svg_tags::builder::*;
}

/// A factory for building [`View`]s.
pub struct ElementBuilder<G: GenericNode, F: FnOnce() -> G + 'static>(F);
impl<G: GenericNode, F: FnOnce() -> G + 'static> std::fmt::Debug for ElementBuilder<G, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElementBuilder").finish()
    }
}

/// A trait that is implemented only for [`ElementBuilder`] and [`View`].
/// This should be considered implementation details and should not be used.
pub trait ElementBuilderOrView<G: GenericNode> {
    /// Convert into a [`View`].
    fn into_view(self) -> View<G>;
}

impl<G: GenericNode> ElementBuilderOrView<G> for View<G> {
    fn into_view(self) -> View<G> {
        self
    }
}

impl<G: GenericNode, F: FnOnce() -> G + 'static> ElementBuilderOrView<G> for ElementBuilder<G, F> {
    fn into_view(self) -> View<G> {
        self.view()
    }
}

/// Construct a new [`ElementBuilder`] from a tag name.
///
/// # Example
/// ```
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// # fn _test1<G: Html>() -> View<G> {
/// tag("a")      // Not recommended. Use `a()` instead.
/// # .view() }
/// # fn _test2<G: Html>() -> View<G> {
/// tag("button") // Not recommended. Use `button()` instead.
/// # .view() }
/// # fn _test3<G: Html>() -> View<G> {
/// tag("my-custom-element")
/// # .view() }
/// // etc...
/// ```
pub fn tag<G: GenericNodeElements>(
    t: impl Into<Cow<'static, str>> + 'static,
) -> ElementBuilder<G, impl FnOnce() -> G> {
    ElementBuilder::new(move || G::element_from_tag(t.into()))
}

//  Implementation note:
//  It might be tempting to extract a common function for all of the functions of the form:
//       let el = (self.0)(); ...
//       el
//  but we used to have one (called `map`) and it was removed on purpose.
//  The trouble is that each call to this internal utility function will result in an expnentially
//  larger type, as the `G` parameter to the previous builder is included twice in the type of the
//  closure (once for the call to the function the user called, like `.attr`, and once for the
//  `.map` call internally). If something causes the `ElementBuilder` to have a nontrivial `Drop` --
//  which happens if any user-supplied or internally generated closure includes a type with a
//  nontrivial `Drop` in its captures -- then the whole exponential type needs to be materialized.
//  This causes both slow compile times and, in some cases, completely breaks `wasm-bindgen`
//  because the generated type can create mangled names on the order of 100s or 1000s of kilobytes
//  in size.
//
//  See:
//    - https://github.com/rust-lang/rust/issues/109363 Rust issue for exponential blowup in mangled
//      function name size
//    - https://github.com/rustwasm/wasm-bindgen/issues/3362 wasm-bindgen issue for falure on
//      mangled functions > 100_000 bytes in length
impl<G: GenericNode, F: FnOnce() -> G + 'static> ElementBuilder<G, F> {
    pub(crate) fn new(f: F) -> Self {
        Self(f)
    }

    /// Set the attribute of the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// button().attr("type", "submit")
    /// # .view() }
    /// ```
    pub fn attr(
        self,
        name: impl Into<Cow<'static, str>> + 'static,
        value: impl Into<Cow<'static, str>> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            el.set_attribute(name.into(), value.into());
            el
        })
    }

    /// Set the boolean attribute of the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// input().bool_attr("required", true)
    /// # .view() }
    /// ```
    pub fn bool_attr(
        self,
        name: impl Into<Cow<'static, str>> + 'static,
        value: bool,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            if value {
                el.set_attribute(name.into(), "".into());
            }
            el
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
    /// # fn _test<G: Html>() -> View<G> {
    /// let input_type = create_signal("text");
    /// input().dyn_attr("type", move || Some(input_type.get()))
    /// # .view() }
    /// ```
    pub fn dyn_attr<S: Into<Cow<'static, str>> + 'static>(
        self,
        name: impl Into<Cow<'static, str>>,
        mut value: impl FnMut() -> Option<S> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        let name = name.into();
        ElementBuilder::new(move || {
            let el_ = (self.0)();
            let el = el_.clone();
            create_effect(move || {
                let value = value();
                if let Some(value) = value {
                    el.set_attribute(name.clone(), value.into());
                } else {
                    el.remove_attribute(name.clone());
                }
            });
            el_
        })
    }

    /// Adds a dynamic boolean attribute on the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// let required = create_signal(true);
    /// input().dyn_bool_attr("required", move || required.get())
    /// # .view() }
    /// ```
    pub fn dyn_bool_attr(
        self,
        name: impl Into<Cow<'static, str>>,
        mut value: impl FnMut() -> bool + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        let name = name.into();
        ElementBuilder::new(move || {
            let el_ = (self.0)();
            let el = el_.clone();
            create_effect(move || {
                if value() {
                    el.set_attribute(name.clone(), "".into());
                } else {
                    el.remove_attribute(name.clone());
                }
            });
            el_
        })
    }

    /// Set the inner html of the element.
    ///
    /// This will clear any children that have been added with `.c()` or `.t()`.
    ///
    /// The html will not be parsed in non-browser environments. This means that accessing methods
    /// such as [`first_child`](GenericNode::first_child) will return `None`.
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// button().dangerously_set_inner_html("<p>Raw HTML!</p>")
    /// # .view() }
    /// ```
    pub fn dangerously_set_inner_html(
        self,
        html: impl Into<Cow<'static, str>> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            el.dangerously_set_inner_html(html.into());
            el
        })
    }

    /// Dynamically set the inner html of the element.
    ///
    /// This will clear any children that have been added with `.c()` or `.t()`.
    ///
    /// The html will not be parsed in non-browser environments. This means that accessing methods
    /// such as [`first_child`](GenericNode::first_child) will return `None`.
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// button().dyn_dangerously_set_inner_html(|| "<p>Raw HTML!</p>")
    /// # .view() }
    /// ```
    pub fn dyn_dangerously_set_inner_html<U>(
        self,
        mut html: impl FnMut() -> U + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static>
    where
        U: Into<Cow<'static, str>> + 'static,
    {
        ElementBuilder::new(move || {
            let el_ = (self.0)();
            let el = el_.clone();
            create_effect(move || {
                el.dangerously_set_inner_html(html().into());
            });
            el_
        })
    }

    /// Adds a class to the element. This is a shorthand for [`Self::attr`] with the `class`
    /// attribute.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// button().class("bg-green-500").t("My button")
    /// # .view() }
    /// ```
    pub fn class(
        self,
        class: impl AsRef<str> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            el.add_class(class.as_ref());
            el
        })
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
    /// # fn _test<G: Html>() -> View<G> {
    /// let checked_class = create_signal(false);
    /// input()
    ///     .attr("type", "checkbox")
    ///     .dyn_class("bg-red-500", move || checked_class.get())
    /// # .view() }
    /// ```
    pub fn dyn_class(
        self,
        class: impl AsRef<str> + 'static,
        mut apply: impl FnMut() -> bool + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el_ = (self.0)();
            let el = el_.clone();
            create_effect(move || {
                if apply() {
                    el.add_class(class.as_ref());
                } else {
                    el.remove_class(class.as_ref());
                }
            });
            el_
        })
    }

    /// Sets the id of an element. This is a shorthand for [`Self::attr`] with the `id` attribute.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// button().id("my-button")
    /// # .view() }
    /// ```
    pub fn id(
        self,
        class: impl Into<Cow<'static, str>> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            el.set_attribute("id".into(), class.into());
            el
        })
    }

    /// Set a property on the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// input().prop("value", "I am the value set.")
    /// # .view() }
    /// ```
    pub fn prop(
        self,
        name: impl AsRef<str> + 'static,
        property: impl Into<G::PropertyType> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            el.set_property(name.as_ref(), &property.into());
            el
        })
    }

    /// Set a dynamic property on the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// let checked = create_signal(false);
    /// input()
    ///     .attr("type", "checkbox")
    ///     .dyn_prop("checked", move || checked.get())
    /// # .view() }
    /// ```
    pub fn dyn_prop<V: Into<G::PropertyType> + 'static>(
        self,
        name: impl AsRef<str> + 'static,
        mut property: impl FnMut() -> V + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el_ = (self.0)();
            let el = el_.clone();
            create_effect(move || {
                el.set_property(name.as_ref(), &property().into());
            });
            el_
        })
    }

    /// Insert a text node under this element. The inserted child is static by default.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// p()
    ///     .t("Hello World!")
    ///     .t("Text nodes can be chained as well.")
    ///     .t("More text...")
    /// # .view() }
    /// ```
    pub fn t(
        self,
        text: impl Into<Cow<'static, str>> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        #[allow(unused_imports)]
        use std::any::TypeId;
        // Only create a text node if we are not hydrating.
        #[cfg(feature = "hydrate")]
        return ElementBuilder::new(|| {
            let el = (self.0)();
            if TypeId::of::<G>() != TypeId::of::<crate::web::HydrateNode>()
                || sycamore_core::hydrate::hydration_completed()
            {
                el.append_child(&G::text_node(text.into()));
            }
            el
        });
        #[cfg(not(feature = "hydrate"))]
        return ElementBuilder::new(|| {
            let el = (self.0)();
            el.append_child(&G::text_node(text.into()));
            el
        });
    }

    /// Adds a dynamic text node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// let name = create_signal("Sycamore");
    /// p()
    ///     .t("Name: ")
    ///     .dyn_t(move || name.get().to_string())
    /// # .view() }
    /// ```
    pub fn dyn_t<S: AsRef<str> + 'static>(
        self,
        f: impl FnMut() -> S + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(|| {
            let el = (self.0)();
            let memo = create_memo(f);
            Self::dyn_c_internal(&el, move || {
                View::new_node(G::text_node(memo.with(|x| x.as_ref().to_string().into())))
            });
            el
        })
    }

    /// Insert a child node under this element. The inserted child is static by default.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// div().c(
    ///     h1().t("I am a child")
    /// )
    /// # .view() }
    /// ```
    pub fn c(
        self,
        c: impl ElementBuilderOrView<G> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(|| {
            let el = (self.0)();
            render::insert(&el, c.into_view(), None, None, true);
            el
        })
    }

    /// Internal implementation for [`Self::dyn_c`] and [`Self::dyn_t`].
    fn dyn_c_internal(el: &G, f: impl FnMut() -> View<G> + 'static) {
        #[allow(unused_imports)]
        use std::any::{Any, TypeId};

        #[cfg(feature = "ssr")]
        if TypeId::of::<G>() == TypeId::of::<crate::web::SsrNode>() {
            // If Server Side Rendering, insert beginning tag for hydration purposes.
            el.append_child(&G::marker_with_text("#".into()));
            // Create end marker. This is needed to make sure that the node is inserted into the
            // right place.
            let end_marker = G::marker_with_text("/".into());
            el.append_child(&end_marker);
            render::insert(
                el,
                View::new_dyn(f),
                None,
                Some(&end_marker),
                true, /* We don't know if this is the only child or not so we pessimistically
                       * set this to true. */
            );
            return;
        }
        #[cfg(feature = "hydrate")]
        if TypeId::of::<G>() == TypeId::of::<crate::web::HydrateNode>() {
            use crate::utils::hydrate::web::*;
            // Get start and end markers.
            let el_hn = <dyn Any>::downcast_ref::<crate::web::HydrateNode>(el).unwrap();
            let initial = get_next_marker(&el_hn.to_web_sys());
            // Do not drop the HydrateNode because it will be cast into a GenericNode.
            let initial = ::std::mem::ManuallyDrop::new(initial);
            // SAFETY: This is safe because we already checked that the type is HydrateNode.
            // __initial is wrapped inside ManuallyDrop to prevent double drop.
            let initial = unsafe { ::std::ptr::read(&initial as *const _ as *const _) };
            render::insert(
                el,
                View::new_dyn(f),
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
        render::insert(el, View::new_dyn(f), None, Some(&marker), true);
    }

    /// Internal implementation for [`Self::dyn_c_scoped`] and [`Self::dyn_if`].
    fn dyn_c_internal_scoped(el: &G, f: impl FnMut() -> View<G> + 'static) {
        #[allow(unused_imports)]
        use std::any::{Any, TypeId};

        #[cfg(feature = "ssr")]
        if TypeId::of::<G>() == TypeId::of::<crate::web::SsrNode>() {
            // If Server Side Rendering, insert beginning tag for hydration purposes.
            el.append_child(&G::marker_with_text("#".into()));
            // Create end marker. This is needed to make sure that the node is inserted into the
            // right place.
            let end_marker = G::marker_with_text("/".into());
            el.append_child(&end_marker);
            render::insert(
                el,
                View::new_dyn_scoped(f),
                None,
                Some(&end_marker),
                true, /* We don't know if this is the only child or not so we
                       * pessimistically set this to true. */
            );
            return;
        }
        #[cfg(feature = "hydrate")]
        if TypeId::of::<G>() == TypeId::of::<crate::web::HydrateNode>() {
            use crate::utils::hydrate::web::*;
            // Get start and end markers.
            let el_hn = <dyn Any>::downcast_ref::<crate::web::HydrateNode>(el).unwrap();
            let initial = get_next_marker(&el_hn.to_web_sys());
            // Do not drop the HydrateNode because it will be cast into a GenericNode.
            let initial = ::std::mem::ManuallyDrop::new(initial);
            // SAFETY: This is safe because we already checked that the type is HydrateNode.
            // __initial is wrapped inside ManuallyDrop to prevent double drop.
            let initial = unsafe { ::std::ptr::read(&initial as *const _ as *const _) };
            render::insert(
                el,
                View::new_dyn_scoped(f),
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
        render::insert(el, View::new_dyn_scoped(f), None, Some(&marker), true);
    }

    /// Adds a dynamic child. Note that most times, [`dyn_if`](Self::dyn_if) can be used instead
    /// which is more ergonomic.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn some_view<G: Html>() -> View<G> { todo!() }
    /// # fn _test<G: Html>() -> View<G> {
    /// let a_view = || some_view();
    /// div().dyn_c(a_view)
    /// # .view() }
    /// ```
    pub fn dyn_c<O: ElementBuilderOrView<G> + 'static>(
        self,
        mut f: impl FnMut() -> O + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            Self::dyn_c_internal(&el, move || f().into_view());
            el
        })
    }

    /// Adds a dynamic, conditional view.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// let visible = create_signal(true);
    /// div().dyn_if(
    ///     move || visible.get(),
    ///     || p().t("Now you see me"),
    ///     || p().t("Now you don't!"),
    /// )
    /// # .view() }
    /// ```
    pub fn dyn_if<O1: ElementBuilderOrView<G>, O2: ElementBuilderOrView<G>>(
        self,
        cond: impl Fn() -> bool + 'static,
        mut then: impl FnMut() -> O1 + 'static,
        mut r#else: impl FnMut() -> O2 + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        let cond = Rc::new(cond);
        ElementBuilder::new(move || {
            let el = (self.0)();
            // FIXME: should be dyn_c_internal_scoped to prevent memory leaks.
            Self::dyn_c_internal(&el, move || {
                if create_selector({
                    let cond = Rc::clone(&cond);
                    #[allow(clippy::redundant_closure)] // FIXME: clippy false positive
                    move || cond()
                })
                .get()
                {
                    then().into_view()
                } else {
                    r#else().into_view()
                }
            });
            el
        })
    }

    /// Adds a dynamic child that is created in a new reactive scope.
    ///
    /// [`dyn_c`](Self::dyn_c) uses [`create_effect`] whereas this method uses
    /// [`create_effect_scoped`].
    pub fn dyn_c_scoped(
        self,
        f: impl FnMut() -> View<G> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(|| {
            let el = (self.0)();
            Self::dyn_c_internal_scoped(&el, f);
            el
        })
    }

    /// Attach an event handler to the element.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// button()
    ///     .t("My button")
    ///     .on(ev::click, |_| web_sys::console::log_1(&"Clicked".into()))
    /// # .view() }
    /// ```
    pub fn on<Ev: EventDescriptor<G::AnyEventData>, S>(
        self,
        ev: Ev,
        handler: impl EventHandler<G::AnyEventData, Ev, S> + 'static,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            el.event(ev, handler);
            el
        })
    }

    /// Get a hold of the raw element by using a [`NodeRef`].
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// let node_ref = create_node_ref();
    /// input().bind_ref(node_ref.clone())
    /// # .view() }
    /// ```
    pub fn bind_ref(self, node_ref: NodeRef<G>) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            node_ref.set(el.clone());
            el
        })
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
    /// fn MyComponent<G: Html>() -> View<G> {
    ///     div()
    ///         /* builder stuff... */
    ///         .view()
    /// }
    /// ```
    pub fn view(self) -> View<G> {
        let el = (self.0)();
        View::new_node(el)
    }
}

/// HTML-specific builder methods.
#[cfg(feature = "web")]
impl<G: Html, F: FnOnce() -> G + 'static> ElementBuilder<G, F> {
    /// Binds a [`Signal`] to the `value` property of the node.
    ///
    /// The [`Signal`] will be automatically updated when the value is updated.
    ///
    /// # Example
    /// ```
    /// # use sycamore::builder::prelude::*;
    /// # use sycamore::prelude::*;
    /// # fn _test<G: Html>() -> View<G> {
    /// let value = create_signal(String::new());
    /// input().bind_value(value)
    /// # .view() }
    /// ```
    pub fn bind_value(
        self,
        sub: Signal<String>,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            #[cfg(target_arch = "wasm32")]
            create_effect({
                let el = el.clone();
                move || sub.with(|sub| el.set_property("value", &sub.as_str().into()))
            });
            el.event(
                ev::input,
                Box::new(move |e: web_sys::Event| {
                    let val = js_sys::Reflect::get(
                        &e.target().expect("missing target on input event"),
                        &"value".into(),
                    )
                    .expect("missing property `value`")
                    .as_string()
                    .expect("value should be a string");
                    sub.set(val);
                }),
            );
            el
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
    /// # fn _test<G: Html>() -> View<G> {
    /// let checked = create_signal(true);
    /// input().attr("type", "checkbox").bind_checked(checked)
    /// # .view() }
    /// ```
    pub fn bind_checked(
        self,
        sub: Signal<bool>,
    ) -> ElementBuilder<G, impl FnOnce() -> G + 'static> {
        ElementBuilder::new(move || {
            let el = (self.0)();
            #[cfg(target_arch = "wasm32")]
            create_effect({
                let el = el.clone();
                let sub = sub.clone();
                move || {
                    el.set_property("checked", &(sub.get()).into());
                }
            });
            el.event(
                ev::change,
                Box::new(move |e: web_sys::Event| {
                    let val = js_sys::Reflect::get(
                        &e.target().expect("missing target on change event"),
                        &"checked".into(),
                    )
                    .expect("missing property `checked`")
                    .as_bool()
                    .expect("could not get property `checked` as a bool");
                    sub.set(val);
                }),
            );
            el
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
/// fn MyComponent<G: Html>() -> View<G> {
///     h1().t("I am a component").view()
/// }
///
/// // Elsewhere...
/// # fn view<G: Html>() -> View<G> {
/// component(|| MyComponent())
/// # }
/// ```
pub fn component<G>(f: impl FnOnce() -> View<G>) -> View<G>
where
    G: GenericNode,
{
    component_scope(f)
}

/// Create a [`View`] from an array of [`View`].
///
/// # Example
/// ```
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// # fn _test<G: Html>() -> View<G> {
/// fragment([
///     div().view(),
///     div().view(),
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
/// # fn _test1<G: Html>() -> View<G> {
/// t("Hello!")
/// # }
/// # fn _test2<G: Html>() -> View<G> {
/// t("This is top level text.")
/// # }
/// # fn _test3<G: Html>() -> View<G> {
/// t("We aren't directly nested under an element.")
/// # }
/// // etc...
/// ```
pub fn t<G: GenericNode>(t: impl Into<Cow<'static, str>>) -> View<G> {
    View::new_node(G::text_node(t.into()))
}

/// Construct a new top-level dynamic text [`View`].
///
/// # Example
/// ```
/// # use sycamore::builder::prelude::*;
/// # use sycamore::prelude::*;
/// # fn _test<G: Html>() -> View<G> {
/// dyn_t(|| "Hello!")
/// # }
/// ```
pub fn dyn_t<G: GenericNode, S: Into<Cow<'static, str>>>(
    mut f: impl FnMut() -> S + 'static,
) -> View<G> {
    View::new_dyn(move || View::new_node(G::text_node(f().into())))
}

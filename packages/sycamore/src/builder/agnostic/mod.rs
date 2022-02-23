//! The renderer-agnostic API.

use js_sys::Reflect;
use std::collections::HashMap;
use std::iter::FromIterator;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::component::component_scope;
use crate::generic_node::{GenericNode, Html};
use crate::noderef::NodeRef;
use crate::reactive::*;
use crate::utils::render;
use crate::view::View;

pub mod prelude {
    pub use super::component;
    pub use super::fragment;
    pub use super::node;
}

/// Create [`NodeBuilder`] to create UI elements.
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # use sycamore::builder::agnostic::node;
/// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
/// node(ctx, "div").build()
/// # }
/// # fn _test2<G: GenericNode>(ctx: ScopeRef) -> View<G> {
/// node(ctx, "a").build()
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
/// ```compile_fail
/// use sycamore::prelude::*;
/// # use sycamore::builder::html::*;
/// #[component]
/// fn MyComponent<G>(ctx: ScopeRef) -> View<G> {
///     h1(ctx).text("I am a component").build()
/// }
///
/// // Elsewhere in another component.
/// # fn view<G: Html>() -> View<G> {
/// // FIXME: how is component() supposed to pass down ctx?
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
/// # use sycamore::builder::agnostic::fragment;
/// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
/// fragment([
///     div(ctx).build(),
///     div(ctx).build()
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    ///  div(ctx)
    ///     .child(h1(ctx).text("I am a child").build())
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// let visible = ctx.create_signal(true);
    ///
    /// div(ctx)
    ///     .dyn_child(
    ///         move || {
    ///             if *visible.get() {
    ///                 h1(ctx).text("I am a child").build()
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// h1(ctx).text("I am text").build()
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// let required = ctx.create_signal(false);
    ///
    /// h1(ctx)
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
    /// ```compile_fail
    /// use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// #[component]
    /// fn MyComponent<G>(ctx: ScopeRef) -> View<G> {
    ///     h1(ctx).text("My component").build()
    /// }
    ///
    /// # fn _test<G: Html>(ctx: ScopeRef) -> View<G> {
    /// // FIXME: how is .component() supposed to pass down ctx?
    /// div(ctx).component::<MyComponent<_>>(()).build()
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// button(ctx).id("my-button").build()
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// button(ctx).attr("type", "submit").build()
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// input(ctx).bool_attr("required", true).build()
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
    /// # use std::ops::Deref;
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// let input_type = ctx.create_signal(Some("text"));
    ///
    /// input(ctx)
    ///     .dyn_attr("type", input_type.deref())
    ///     .build()
    /// }
    /// ```
    pub fn dyn_attr<N, T>(&self, name: N, value: &'a ReadSignal<Option<T>>) -> &Self
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
    /// # use std::ops::Deref;
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// let required = ctx.create_signal(true);
    ///
    /// input(ctx)
    ///     .dyn_bool_attr("required", required.deref()).build()
    /// }
    /// ```
    pub fn dyn_bool_attr<N>(&self, name: N, value: &'a ReadSignal<bool>) -> &Self
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// input(ctx).prop("value", "I am the value set.").build()
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
    /// # use std::ops::Deref;
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// let checked = ctx.create_signal(Some(false));
    ///
    /// input(ctx)
    ///     .attr("type", "checkbox")
    ///     .dyn_prop("checked", checked.deref())
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// button(ctx).class("bg-green-500").text("My Button").build()
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
    /// # use std::ops::Deref;
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// let checked_class = ctx.create_signal(false);
    ///
    /// input(ctx)
    ///     .attr("type", "checkbox")
    ///     .dyn_class("bg-red-500", checked_class.deref())
    ///     .build()
    /// }
    /// ```
    pub fn dyn_class(&self, class: impl ToString, apply: &'a ReadSignal<bool>) -> &Self {
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// button(ctx)
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// let node_ref = NodeRef::new();
    ///
    /// input(ctx)
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
    /// # fn _test<G: GenericNode>(ctx: ScopeRef) -> View<G> {
    /// input(ctx)
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
    /// # fn _test<G: Html>(ctx: ScopeRef) -> View<G> {
    /// let value = ctx.create_signal(String::new());
    ///
    /// input(ctx)
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
    /// # fn _test<G: Html>(ctx: ScopeRef) -> View<G> {
    /// let checked = ctx.create_signal(false);
    ///
    /// input(ctx)
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

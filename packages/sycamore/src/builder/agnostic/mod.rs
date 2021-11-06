//! The renderer-agnostic API.

use crate::component::Component;
use crate::generic_node::{GenericNode, Html};
use crate::noderef::NodeRef;
use crate::template::Template;
use crate::utils::render;
use js_sys::Reflect;
use std::collections::HashMap;
use std::iter::FromIterator;
use sycamore_reactive::{cloned, create_effect, create_memo, Signal, StateHandle};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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
/// # fn _test<G: GenericNode>() -> Template<G> {
/// node("div").build()
/// # }
/// # fn _test2<G: GenericNode>() -> Template<G> {
/// node("a").build()
/// # }
/// ```
pub fn node<G>(tag: &'static str) -> NodeBuilder<G>
where
    G: GenericNode,
{
    NodeBuilder {
        element: G::element(tag),
    }
}

/// Instantiate a component as a [`Template`].
///
/// # Example
/// ```
/// use sycamore::prelude::*;
/// # use sycamore::builder::html::*;
/// #[component(MyComponent<G>)]
/// fn my_component() -> Template<G> {
///     h1().text("I am a component").build()
/// }
///
/// // Elsewhere in another component.
/// # fn view<G: Html>() -> Template<G> {
/// component::<_, MyComponent<_>>(())
/// # }
/// ```
pub fn component<G, C>(props: C::Props) -> Template<G>
where
    G: GenericNode + Html,
    C: Component<G>,
{
    C::__create_component(props)
}

/// Create a [`Template`] from an array of [`Template`].
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # use sycamore::builder::html::*;
/// # fn _test<G: GenericNode>() -> Template<G> {
/// fragment([
///     div().build(),
///     div().build()
/// ])
/// # }
/// ```
pub fn fragment<G, const N: usize>(parts: [Template<G>; N]) -> Template<G>
where
    G: GenericNode,
{
    Template::new_fragment(Vec::from_iter(parts.to_vec()))
}

/// The main type powering the builder API.
#[derive(Debug)]
pub struct NodeBuilder<G>
where
    G: GenericNode,
{
    element: G,
}

impl<G> NodeBuilder<G>
where
    G: GenericNode,
{
    /// Add a child [`Template`].
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
    ///  div()
    ///     .child(h1().text("I am a child").build())
    ///     .build()
    /// # }
    /// ```
    pub fn child(&self, child: Template<G>) -> &Self {
        render::insert(&self.element, child, None, None, true);

        self
    }

    /// Add a dynamic child [`Template`]
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
    /// let visible = Signal::new(true);
    ///
    /// div()
    ///     .dyn_child(
    ///         move || {
    ///             if *visible.get() {
    ///                 h1().text("I am a child").build()
    ///             } else { Template::empty() }
    ///         }
    ///     )
    ///     .build()
    /// # }
    /// ```
    pub fn dyn_child(&self, child: impl FnMut() -> Template<G> + 'static) -> &Self {
        render::insert(&self.element, Template::new_dyn(child), None, None, true);

        self
    }

    /// Adds a text node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
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
    /// # fn _test<G: GenericNode>() -> Template<G> {
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
        F: FnMut() -> O + 'static,
        O: AsRef<str> + 'static,
    {
        let memo = create_memo(text);

        self.dyn_child(move || Template::new_node(G::text_node(memo.get().as_ref().as_ref())));

        self
    }

    /// Renders a component as a child.
    ///
    /// # Example
    /// ```
    /// use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// #[component(MyComponent<G>)]
    /// fn my_component() -> Template<G> {
    ///     h1().text("My component").build()
    /// }
    ///
    /// # fn _test<G: Html>() -> Template<G> {
    /// div().component::<MyComponent<_>>(()).build()
    /// }
    /// ```
    pub fn component<C>(&self, props: C::Props) -> &Self
    where
        C: Component<G>,
    {
        self.child(C::__create_component(props));

        self
    }

    /// Convenience function for adding an `id` to a node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
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
    /// # fn _test<G: GenericNode>() -> Template<G> {
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
    /// # fn _test<G: GenericNode>() -> Template<G> {
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
    /// # fn _test<G: GenericNode>() -> Template<G> {
    /// let input_type = Signal::new(Some("text"));
    ///
    /// input()
    ///     .dyn_attr("type", input_type.handle())
    ///     .build()
    /// }
    /// ```
    pub fn dyn_attr<N, T>(&self, name: N, value: StateHandle<Option<T>>) -> &Self
    where
        N: ToString,
        T: ToString,
    {
        let element = self.element.clone();

        let name = name.to_string();

        cloned!((name) => create_effect(move || {
            let v = value.get();

            if let Some(v) = &*v {
                element.set_attribute(name.as_ref(), v.to_string().as_ref());
            } else {
                element.remove_attribute(name.as_ref());
            }
        }));

        self
    }

    /// Adds a dynamic boolean attribute on the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
    /// let required = Signal::new(true);
    ///
    /// input()
    ///     .dyn_bool_attr("required", required.handle()).build()
    /// }
    /// ```
    pub fn dyn_bool_attr<N>(&self, name: N, value: StateHandle<bool>) -> &Self
    where
        N: ToString,
    {
        let element = self.element.clone();

        let name = name.to_string();

        cloned!((name) => create_effect(move || {
            let v = value.get();

            if *v {
                element.set_attribute(name.as_ref(), "");
            } else {
                element.remove_attribute(name.as_ref());
            }
        }));

        self
    }

    /// Set a property on the node.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
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
    /// # fn _test<G: GenericNode>() -> Template<G> {
    /// let checked = Signal::new(Some(false));
    ///
    /// input()
    ///     .attr("type", "checkbox")
    ///     .dyn_prop("checked", checked.handle())
    ///     .build()
    /// }
    /// ```
    pub fn dyn_prop<N, T>(&self, name: N, value: StateHandle<Option<T>>) -> &Self
    where
        N: ToString,
        T: ToString,
    {
        let element = self.element.clone();

        let name = name.to_string();

        create_effect(move || {
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
    /// # fn _test<G: GenericNode>() -> Template<G> {
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
    /// # fn _test<G: GenericNode>() -> Template<G> {
    /// let checked_class = Signal::new(false);
    ///
    /// input()
    ///     .attr("type", "checkbox")
    ///     .dyn_class("bg-red-500", checked_class.handle())
    ///     .build()
    /// }
    /// ```
    pub fn dyn_class(&self, class: impl ToString, apply: StateHandle<bool>) -> &Self {
        let class = class.to_string();
        let element = self.element.clone();

        create_effect(move || {
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
    /// # fn _test<G: GenericNode>() -> Template<G> {
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
        H: Fn(web_sys::Event) + 'static,
    {
        self.element.event(event.as_ref(), Box::new(handler));

        self
    }

    /// Binds `sub` to the `value` property of the node.
    ///
    /// `sub` will be automatically updated when the value is updated.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
    /// let value = Signal::new(String::new());
    ///
    /// input()
    ///     .bind_value(value)
    ///     .build()
    /// # }
    /// ```
    pub fn bind_value(&self, sub: Signal<String>) -> &Self {
        let sub_handle = create_memo(cloned!((sub) => move || {
            Some((*sub.get()).clone())
        }));

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
    /// # fn _test<G: GenericNode>() -> Template<G> {
    /// let checked = Signal::new(false);
    ///
    /// input()
    ///     .attr("type", "checkbox")
    ///     .bind_checked(checked)
    ///     .build()
    /// # }
    /// ```
    pub fn bind_checked(&self, sub: Signal<bool>) -> &Self {
        let sub_handle = create_memo(cloned!((sub) => move || {
            Some(*sub.get())
        }));

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

    /// Get a hold of the [`element`](::web_sys::Node) by using a [`NodeRef`].
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
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

    /// Builds the [`NodeBuilder`] and returns a [`Template`].
    ///
    /// This is the function that should be called at the end of the node
    /// building chain.
    ///
    /// # Example
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::builder::html::*;
    /// # fn _test<G: GenericNode>() -> Template<G> {
    /// input()
    ///     /* builder stuff */
    ///     .build()
    /// # }
    /// ```
    pub fn build(&self) -> Template<G> {
        Template::new_node(self.element.to_owned())
    }
}

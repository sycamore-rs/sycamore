use crate::component::Component;
use crate::generic_node::GenericNode;
use crate::noderef::NodeRef;
use crate::template::Template;
use crate::utils::render;
use js_sys::Reflect;
use std::collections::HashMap;
use sycamore_reactive::{cloned, create_effect, create_memo, Signal, StateHandle};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub mod prelude {
    pub use super::component;
    pub use super::node;
}

pub fn node<G>(tag: &'static str) -> NodeBuilder<G>
where
    G: GenericNode,
{
    NodeBuilder {
        element: G::element(tag),
    }
}

pub fn component<G, C>(props: C::Props) -> Template<G>
where
    G: GenericNode,
    C: Component<G>,
{
    C::__create_component(props)
}

#[cfg_attr(debug_assertions, derive(Debug))]
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
    pub fn child(&self, child: Template<G>) -> &Self {
        render::insert(&self.element, child, None, None, true);

        self
    }

    pub fn dyn_child(&self, child: impl FnMut() -> Template<G> + 'static) -> &Self {
        render::insert(&self.element, Template::new_dyn(child), None, None, true);

        self
    }

    pub fn only_child(&self, child: Template<G>) -> &Self {
        render::insert(&self.element, child, None, None, false);

        self
    }

    pub fn dyn_only_child(&self, child: impl FnMut() -> Template<G> + 'static) -> &Self {
        render::insert(&self.element, Template::new_dyn(child), None, None, false);

        self
    }

    pub fn text(&self, text: impl AsRef<str>) -> &Self {
        self.element.append_child(&G::text_node(text.as_ref()));

        self
    }

    pub fn dyn_text<F, O>(&self, text: F) -> &Self
    where
        F: FnMut() -> O + 'static,
        O: AsRef<str> + 'static,
    {
        let memo = create_memo(text);

        self.dyn_child(move || Template::new_node(G::text_node(memo.get().as_ref().as_ref())));

        self
    }

    pub fn component<C>(&self, props: C::Props) -> &Self
    where
        C: Component<G>,
    {
        self.child(C::__create_component(props));

        self
    }

    pub fn id(&self, id: impl AsRef<str>) -> &Self {
        self.attr("id", id.as_ref())
    }

    pub fn attr<N, Va>(&self, name: N, value: Va) -> &Self
    where
        N: AsRef<str>,
        Va: AsRef<str>,
    {
        self.element.set_attribute(name.as_ref(), value.as_ref());

        self
    }

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

    pub fn prop<N, Va>(&self, name: N, property: Va) -> &Self
    where
        N: AsRef<str>,
        Va: Into<JsValue>,
    {
        self.element.set_property(name.as_ref(), &property.into());

        self
    }

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

    pub fn class(&self, class: impl ToString) -> &Self {
        self.element.add_class(class.to_string().as_ref());

        self
    }

    pub fn add_dyn_class(&self, class: impl ToString, apply: StateHandle<bool>) -> &Self {
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

    pub fn styles(&self, styles: HashMap<String, String>) -> &Self {
        let styles = styles
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(";");

        self.attr("style", styles);

        self
    }

    // Need the ability to get attributes so one can filter out the
    // applied attribute to add/remove it.
    pub fn dyn_style(
        &self,
        _style: (impl ToString, impl ToString),
        _apply: StateHandle<bool>,
    ) -> &Self {
        todo!("Implement set_dyn_style");
    }

    pub fn event_listener<E, H>(&self, event: E, handler: H) -> &Self
    where
        E: AsRef<str>,
        H: Fn(web_sys::Event) + 'static,
    {
        self.element.event(event.as_ref(), Box::new(handler));

        self
    }

    // We need to store the closure somewhere we can retrieve so we can pass it
    // to remove_event_listener which does not drop the closure, hence, can be
    // added again when needed.
    pub fn dyn_event_listener<E, H>(
        &self,
        _event: E,
        _handler: H,
        _listen: StateHandle<bool>,
    ) -> &Self
    where
        E: AsRef<str>,
        H: Fn(web_sys::Event) + 'static,
    {
        todo!("Implement dyn event listener");
    }

    pub fn bind_value(&self, sub: Signal<String>) -> &Self {
        let sub_handle = create_memo(cloned!((sub) => move || {
            Some((*sub.get()).clone())
        }));

        self.dyn_prop("value", sub_handle);

        self.event_listener("input", move |e| {
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

    pub fn bind_checked(&self, sub: Signal<bool>) -> &Self {
        let sub_handle = create_memo(cloned!((sub) => move || {
            Some((*sub.get()).clone())
        }));

        self.dyn_prop("checked", sub_handle);

        self.event_listener("change", move |e| {
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

    pub fn bind_ref(&self, node_ref: NodeRef<G>) -> &Self {
        node_ref.set(self.element.clone());

        self
    }

    pub fn build(&self) -> Template<G> {
        Template::new_node(self.element.to_owned())
    }
}

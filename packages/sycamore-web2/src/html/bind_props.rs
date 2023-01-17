//! Definitions for properties that can be two-way binded to with the [`bind`] directive.

use sycamore_core2::attributes::ApplyAttr;
use sycamore_core2::elements::{AnyElement, TypedElement};
use sycamore_reactive::{create_effect, Scope, Signal};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Event;

use super::events::{on, OnAttr};
use crate::render::{get_render_env, RenderEnv};
use crate::web_node::WebNode;

/// Attribute directive for creating a two-way binding between a property value and a signal.
#[allow(non_camel_case_types)]
pub struct bind;

pub struct BindAttr<T, U> {
    prop: &'static str,
    ev: OnAttr<U>,
    _marker: std::marker::PhantomData<T>,
}

impl<T, U> BindAttr<T, U> {
    pub const fn new(prop: &'static str, ev: OnAttr<U>) -> Self {
        Self {
            prop,
            ev,
            _marker: std::marker::PhantomData,
        }
    }
}

fn bind_to_element<'a, Ev: From<JsValue> + Into<JsValue>, T>(
    cx: Scope<'a>,
    el: &WebNode,
    prop: &'static str,
    ev: OnAttr<Ev>,
    signal: &'a Signal<T>,
    into: impl Fn(&T) -> JsValue + 'a,
    from: impl Fn(&JsValue) -> Option<T> + 'a,
) {
    if get_render_env(cx) == RenderEnv::Dom {
        ApplyAttr::<WebNode, _, AnyElement>::apply(ev, cx, el, move |ev: Ev| {
            let ev: JsValue = ev.into();
            let target = ev.unchecked_into::<Event>().target().unwrap();
            let prop = js_sys::Reflect::get(&target, &prop.into()).unwrap();
            signal.set(from(&prop).unwrap());
        });
        let el = el.clone();
        create_effect(cx, move || {
            let value = signal.get();
            el.set_property(prop, into(&value));
        });
    }
}

impl<'a, E: TypedElement<WebNode>, Ev: From<JsValue> + Into<JsValue>>
    ApplyAttr<'a, WebNode, &'a Signal<bool>, E> for BindAttr<bool, Ev>
{
    const NEEDS_HYDRATE: bool = true;
    fn apply(self, cx: Scope<'a>, el: &WebNode, value: &'a Signal<bool>) {
        bind_to_element(
            cx,
            el,
            self.prop,
            self.ev,
            value,
            |&rs| rs.into(),
            JsValue::as_bool,
        );
    }
}

impl<'a, E: TypedElement<WebNode>, Ev: From<JsValue> + Into<JsValue>>
    ApplyAttr<'a, WebNode, &'a Signal<f64>, E> for BindAttr<f64, Ev>
{
    const NEEDS_HYDRATE: bool = true;
    fn apply(self, cx: Scope<'a>, el: &WebNode, value: &'a Signal<f64>) {
        bind_to_element(
            cx,
            el,
            self.prop,
            self.ev,
            value,
            |&rs| rs.into(),
            JsValue::as_f64,
        );
    }
}

impl<'a, E: TypedElement<WebNode>, Ev: From<JsValue> + Into<JsValue>>
    ApplyAttr<'a, WebNode, &'a Signal<String>, E> for BindAttr<String, Ev>
{
    const NEEDS_HYDRATE: bool = true;
    fn apply(self, cx: Scope<'a>, el: &WebNode, value: &'a Signal<String>) {
        bind_to_element(
            cx,
            el,
            self.prop,
            self.ev,
            value,
            |rs| rs.into(),
            JsValue::as_string,
        );
    }
}

#[allow(non_upper_case_globals)]
impl bind {
    /// Creates a two-way binding to an input's `value`.
    pub const value: BindAttr<String, Event> = BindAttr::new("value", on::input);
    /// Same as [`bind`] but automatically converts the value to a [`f64`].
    pub const value_as_number: BindAttr<f64, Event> = BindAttr::new("value", on::input);
    /// Creates a two-way binding to a checkbox's `checked` state.
    pub const checked: BindAttr<bool, Event> = BindAttr::new("checked", on::change);
}

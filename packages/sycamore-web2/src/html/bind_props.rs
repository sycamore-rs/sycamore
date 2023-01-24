//! Definitions for properties that can be two-way binded to with the [`bind`] directive.

#![allow(non_upper_case_globals)]

use sycamore_core2::elements::{AsNode, TypedElement};
use sycamore_reactive::{create_effect, Signal};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Event;

use super::events::{on, OnAttr};
use super::OnAttributes;
use crate::render::{get_render_env, RenderEnv};
use crate::web_node::WebNode;
use crate::ElementBuilder;

pub trait BindAttributes<'a> {
    fn bind<T: JsValueCastToType, U: From<JsValue> + Into<JsValue>>(
        self,
        attr: BindAttr<T, U>,
        signal: &'a Signal<T>,
    ) -> Self;
}

impl<'a, E: TypedElement<WebNode>> BindAttributes<'a> for ElementBuilder<'a, E> {
    fn bind<T: JsValueCastToType, U: From<JsValue> + Into<JsValue>>(
        mut self,
        attr: BindAttr<T, U>,
        signal: &'a Signal<T>,
    ) -> Self {
        self.mark_dyn();
        if get_render_env(self.cx()) == RenderEnv::Ssr {
            return self;
        }
        let el = self.as_node().clone();
        create_effect(self.cx(), move || {
            let value = signal.get();
            el.set_property(attr.prop, T::cast_into(&value));
        });
        self.on(attr.ev, move |ev: U| {
            let ev: JsValue = ev.into();
            let target = ev.unchecked_into::<Event>().target().unwrap();
            let value = js_sys::Reflect::get(&target, &attr.prop.into()).unwrap();
            signal.set(T::cast_from(&value).unwrap()); // TODO: don't unwrap here
        })
    }
}

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

pub trait JsValueCastToType {
    fn cast_from(value: &JsValue) -> Option<Self>
    where
        Self: Sized;
    fn cast_into(&self) -> JsValue;
}

impl JsValueCastToType for bool {
    fn cast_from(value: &JsValue) -> Option<Self> {
        value.as_bool()
    }
    fn cast_into(&self) -> JsValue {
        JsValue::from(*self)
    }
}
impl JsValueCastToType for String {
    fn cast_from(value: &JsValue) -> Option<Self> {
        value.as_string()
    }
    fn cast_into(&self) -> JsValue {
        JsValue::from(self)
    }
}

impl bind {
    /// Creates a two-way binding to an input's `value`.
    pub const value: BindAttr<String, Event> = BindAttr::new("value", on::input);
    /// Same as [`bind`] but automatically converts the value to a [`f64`].
    pub const value_as_number: BindAttr<f64, Event> = BindAttr::new("value", on::input);
    /// Creates a two-way binding to a checkbox's `checked` state.
    pub const checked: BindAttr<bool, Event> = BindAttr::new("checked", on::change);
}

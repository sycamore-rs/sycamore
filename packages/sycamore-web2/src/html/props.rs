//! Definitions for properties that can be used with the [`prop`] directive.

use sycamore_core2::attributes::{ApplyAttr, ApplyAttrDyn};
use sycamore_reactive::{create_effect, Scope};
use wasm_bindgen::JsValue;

use crate::web_node::WebNode;

/// Attribute directive for setting a JS property on an element.
#[allow(non_camel_case_types)]
pub struct prop;

pub struct PropAttr {
    name: &'static str,
}

impl<'a, T: Into<JsValue>> ApplyAttr<'a, WebNode, T> for PropAttr {
    fn apply(self, _cx: Scope<'a>, el: &WebNode, value: T) {
        el.set_property(self.name, value.into());
    }
}

impl<'a, T: Into<JsValue> + 'a> ApplyAttrDyn<'a, WebNode, T> for PropAttr {
    fn apply(self, cx: Scope<'a>, el: &WebNode, mut value: Box<dyn FnMut() -> T + 'a>) {
        let el = el.clone();
        create_effect(cx, move || {
            el.set_property(self.name, value().into());
        });
    }
}

impl prop {
    pub fn custom(name: &'static str) -> PropAttr {
        PropAttr { name }
    }
}

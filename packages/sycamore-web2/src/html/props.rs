//! Definitions for properties that can be used with the [`prop`] directive.

use sycamore_core2::attributes::{ApplyAttr, ApplyAttrDyn};
use sycamore_core2::elements::TypedElement;
use sycamore_reactive::{create_effect, Scope};
use wasm_bindgen::JsValue;

use crate::web_node::WebNode;

/// Attribute directive for setting a JS property on an element.
#[allow(non_camel_case_types)]
pub struct prop;

pub struct PropAttr {
    name: &'static str,
}

impl<'a, T: Into<JsValue>, E: TypedElement<WebNode>> ApplyAttr<'a, WebNode, T, E> for PropAttr {
    const NEEDS_HYDRATE: bool = true;
    fn apply(self, _cx: Scope<'a>, el: &WebNode, value: T) {
        el.set_property(self.name, value.into());
    }
}

impl<'a, T: Into<JsValue> + 'a, E: TypedElement<WebNode>> ApplyAttrDyn<'a, WebNode, T, E>
    for PropAttr
{
    fn apply_dyn(self, cx: Scope<'a>, el: &WebNode, mut value: Box<dyn FnMut() -> T + 'a>) {
        let el = el.clone();
        create_effect(cx, move || {
            el.set_property(self.name, value().into());
        });
    }
}

impl prop {
    pub const fn custom(name: &'static str) -> PropAttr {
        PropAttr { name }
    }
}

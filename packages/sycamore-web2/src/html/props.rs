//! Definitions for properties that can be used with the [`prop`] directive.

use sycamore_core2::elements::TypedElement;
use wasm_bindgen::JsValue;

use crate::web_node::WebNode;
use crate::ElementBuilder;

pub trait PropAttributes {
    fn prop(self, attr: PropAttr, value: impl Into<JsValue>) -> Self;
}

impl<'a, E: TypedElement<WebNode>> PropAttributes for ElementBuilder<'a, E> {
    fn prop(self, attr: PropAttr, value: impl Into<JsValue>) -> Self {
        self.as_node().set_property(attr.name, value.into());
        self
    }
}

/// Attribute directive for setting a JS property on an element.
#[allow(non_camel_case_types)]
pub struct prop;

pub struct PropAttr {
    name: &'static str,
}

impl prop {
    pub const fn custom(name: &'static str) -> PropAttr {
        PropAttr { name }
    }
}

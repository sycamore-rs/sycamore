//! Definitions for properties that can be used with the [`prop`] directive.

use wasm_bindgen::JsValue;

use super::{Attributes, WebElement};
use crate::ElementBuilder;

pub trait PropAttributes<'a> {
    fn prop(self, attr: PropAttr, value: impl Into<JsValue> + 'a) -> Self;
}

impl<'a, E: WebElement> PropAttributes<'a> for ElementBuilder<'a, E> {
    fn prop(self, attr: PropAttr, value: impl Into<JsValue> + 'a) -> Self {
        self.as_node().set_property(attr.name, value.into());
        self
    }
}
impl<'a, E: WebElement> PropAttributes<'a> for Attributes<'a, E> {
    fn prop(self, attr: PropAttr, value: impl Into<JsValue> + 'a) -> Self {
        self.add_fn(|builder| {
            builder.prop(attr, value);
        });
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

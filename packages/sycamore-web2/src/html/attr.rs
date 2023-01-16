//! Definition for HTML attributes that can be used with the [`attr`] directive or with nothing at
//! all (which uses [`attr`] by default).

use sycamore_core2::attributes::{ApplyAttr, ApplyAttrDyn};
use sycamore_core2::generic_node::GenericNode;
use sycamore_reactive::{create_effect, Scope};

use crate::web_node::WebNode;

#[allow(non_camel_case_types)]
pub struct attr;

pub struct BaseAttr {
    name: &'static str,
}

impl BaseAttr {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl<'a, T: ToString> ApplyAttr<'a, WebNode, T> for BaseAttr {
    fn apply(self, _cx: Scope<'a>, el: &WebNode, value: T) {
        el.set_attribute(self.name.into(), value.to_string().into());
    }
}

impl<'a, T: ToString + 'a> ApplyAttrDyn<'a, WebNode, T> for BaseAttr {
    fn apply_dyn(self, cx: Scope<'a>, el: &WebNode, mut value: Box<dyn FnMut() -> T + 'a>) {
        let el = el.clone();
        create_effect(cx, move || {
            el.set_attribute(self.name.into(), value().to_string().into());
        });
    }
}

#[allow(non_upper_case_globals)]
impl attr {
    pub const class: BaseAttr = BaseAttr::new("class");
}

//! HTML and SVG tag definitions.
//!
//! _Documentation sources: <https://developer.mozilla.org/en-US/>_

pub mod elements;

mod attributes;
mod bind_props;
mod events;
mod props;

use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;

pub use attributes::{GlobalAttributes, HtmlGlobalAttributes, SvgGlobalAttributes};
pub use bind_props::{bind, BindAttributes};
pub use events::{on, OnAttributes};
pub use props::{prop, PropAttributes};
use sycamore_core::elements::Spread;
use sycamore_core::generic_node::GenericNode;
use sycamore_reactive::Scope;

use crate::web_node::WebNode;
use crate::ElementBuilder;

use self::elements::WebElement;

type AttrFn<'a, E> = Box<dyn FnOnce(ElementBuilder<'a, E>) + 'a>;

/// A struct that can keep track of the attributes that are added.
/// This can be used as a prop to a component to allow the component to accept arbitrary attributes
/// and then spread them onto the element.
pub struct Attributes<'a, E: WebElement> {
    fns: RefCell<Vec<AttrFn<'a, E>>>,
}

impl<'a, E: WebElement> Attributes<'a, E> {
    /// Create a new instance of [`Attributes`].
    pub fn new() -> Self {
        Self {
            fns: RefCell::new(Vec::new()),
        }
    }

    /// Add a closure.
    pub fn add_fn<F>(&self, f: F)
    where
        F: FnOnce(ElementBuilder<'a, E>) + 'a,
    {
        self.fns.borrow_mut().push(Box::new(f));
    }

    /// Apply all the attributes to the element builder.
    pub fn apply(self, builder: ElementBuilder<'a, E>) {
        for f in self.fns.into_inner() {
            f(builder.clone());
        }
    }
}

impl<'a, E: WebElement> Default for Attributes<'a, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E: WebElement> fmt::Debug for Attributes<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attributes").finish()
    }
}

impl<'a, E: WebElement> Spread<'a, E, WebNode> for Attributes<'a, E> {
    fn spread(self, cx: Scope<'a>, el: &WebNode) {
        self.apply(ElementBuilder::from_element(cx, E::from_node(el.clone())));
    }
}

/// Something that can have attributes.
pub trait SetAttribute {
    fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>);
    fn remove_attribute(&self, name: Cow<'static, str>);
}

impl<'a, E: WebElement> SetAttribute for ElementBuilder<'a, E> {
    fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>) {
        self.as_node().set_attribute(name, value);
    }

    fn remove_attribute(&self, name: Cow<'static, str>) {
        self.as_node().remove_attribute(name);
    }
}
impl<'a, E: WebElement> SetAttribute for Attributes<'a, E> {
    fn set_attribute(&self, name: Cow<'static, str>, value: Cow<'static, str>) {
        self.fns.borrow_mut().push(Box::new(move |builder| {
            builder.set_attribute(name.clone(), value.clone());
        }));
    }

    fn remove_attribute(&self, name: Cow<'static, str>) {
        self.fns.borrow_mut().push(Box::new(move |builder| {
            builder.remove_attribute(name.clone());
        }));
    }
}

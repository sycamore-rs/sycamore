//! Definitions for events that can be used with the [`on`] directive.

use sycamore_core2::attributes::ApplyAttr;
use sycamore_core2::generic_node::GenericNodeElements;
use sycamore_reactive::Scope;
use wasm_bindgen::JsValue;
use web_sys::MouseEvent;

use crate::web_node::WebNode;

/// Attribute directive for attaching an event listener to an element.
#[allow(non_camel_case_types)]
pub struct on;

/// A trait that is implemented for all event handlers.
///
/// The type generic `T` is the type of the event data.
/// The type generic `S` is a dummy generic so that the trait can be implemented on both normal
/// functions and async functions.
pub trait EventHandler<'a, T, S> {
    fn call(&mut self, cx: Scope<'a>, event: T);
}

impl<'a, T, F> EventHandler<'a, T, ()> for F
where
    F: FnMut(T) + 'a,
{
    fn call(&mut self, _cx: Scope<'a>, event: T) {
        self(event)
    }
}

#[cfg(feature = "suspense")]
impl<'a, T, F, Fut> EventHandler<'a, T, ((), ())> for F
where
    F: FnMut(T) -> Fut,
    Fut: std::future::Future<Output = ()> + 'a,
{
    fn call(&mut self, cx: Scope<'a>, event: T) {
        sycamore_futures::spawn_local_scoped(cx, self(event));
    }
}

/// Describes data about an event.
///
/// The `T` generic is the type of the event data.
pub struct OnAttr<T> {
    name: &'static str,
    _marker: std::marker::PhantomData<T>,
}

impl<T> OnAttr<T> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T: From<JsValue>, F: FnMut(T) + 'a> ApplyAttr<'a, WebNode, F> for OnAttr<T> {
    fn apply(self, cx: Scope<'a>, el: &WebNode, mut value: F) {
        let type_erased = Box::new(move |ev: JsValue| value(ev.into()));
        el.add_event_listener(cx, self.name, type_erased);
    }
}

#[allow(non_upper_case_globals)]
impl on {
    pub const click: OnAttr<MouseEvent> = OnAttr::new("click");
}

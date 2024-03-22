//! Internal utilities for Sycamore.
//!
//! # Stability
//! This API is considered implementation details and should not at any time be considered stable.
//! The API can change without warning and without a semver compatible release.

/// Utilities for hydration support.
#[cfg(feature = "hydrate")]
pub mod hydrate {
    pub use sycamore_core::hydrate::*;
    pub use sycamore_web::hydrate as web;
}

use std::borrow::Cow;

use js_sys::Reflect;
use sycamore_core::event::{EventDescriptor, EventHandler};
pub use sycamore_core::render;
use wasm_bindgen::JsValue;

use crate::prelude::*;
use crate::rt::Event;
use crate::web::html::ev;

/// If `el` is a `HydrateNode`, use `get_next_marker` to get the initial node value.
pub fn initial_node<G: GenericNode>(_el: &G) -> Option<View<G>> {
    #[cfg(feature = "hydrate")]
    {
        use std::any::Any;
        use std::mem::ManuallyDrop;
        use std::ptr;

        if let Some(el) = <dyn Any>::downcast_ref::<HydrateNode>(_el) {
            let initial = hydrate::web::get_next_marker(&el.to_web_sys());
            // Do not drop the HydrateNode because it will be cast into a GenericNode.
            let initial = ManuallyDrop::new(initial);
            // SAFETY: This is safe because we already checked that the type is HydrateNode.
            // initial is wrapped inside ManuallyDrop to prevent double drop.
            unsafe { ptr::read(&initial as *const _ as *const _) }
        } else {
            None
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        None
    }
}

#[doc(hidden)]
pub fn erase_handler<Ev, Handler, S>(mut handler: Handler) -> Box<dyn FnMut(JsValue) + 'static>
where
    Handler: EventHandler<JsValue, Ev, S> + 'static,
    Ev: EventDescriptor<JsValue>,
{
    Box::new(move |e: JsValue| handler.call(e.into()))
}

/// Apply an `AttributeValue` to an element. Used by the `view!` macro.
pub fn apply_attribute<G: GenericNode<AnyEventData = JsValue, PropertyType = JsValue>>(
    el: G,
    name: Cow<'static, str>,
    value: AttributeValue<G>,
) {
    match value {
        AttributeValue::Str(s) => {
            el.set_attribute(name.clone(), Cow::Borrowed(s));
        }
        AttributeValue::DynamicStr(mut s) => {
            create_effect({
                let name = name.clone();
                move || el.set_attribute(name.clone(), Cow::Owned(s()))
            });
        }
        AttributeValue::Bool(value) => {
            if value {
                el.set_attribute(name.clone(), Cow::Borrowed(""));
            }
        }
        AttributeValue::DynamicBool(mut value) => {
            create_effect({
                let name = name.clone();
                move || {
                    if value() {
                        el.set_attribute(name.clone(), Cow::Borrowed(""));
                    } else {
                        el.remove_attribute(name.clone());
                    }
                }
            });
        }
        AttributeValue::DangerouslySetInnerHtml(value) => {
            el.dangerously_set_inner_html(value.into());
        }
        AttributeValue::DynamicDangerouslySetInnerHtml(value) => {
            create_effect(move || {
                el.dangerously_set_inner_html(Cow::Owned(value.to_string()));
            });
        }
        AttributeValue::Event(event, handler) => {
            el.untyped_event(Cow::Borrowed(event), handler);
        }
        AttributeValue::BindBool(prop, signal) => {
            #[cfg(target_arch = "wasm32")]
            {
                create_effect({
                    let signal = signal.clone();
                    let el = el.clone();
                    move || el.set_property(prop, &JsValue::from_bool(signal.get()))
                });
            }
            el.event(ev::change, {
                move |event: Event| {
                    signal.set(
                        JsValue::as_bool(
                            &Reflect::get(&event.target().unwrap(), &prop.into()).unwrap(),
                        )
                        .unwrap(),
                    );
                }
            });
        }
        AttributeValue::BindNumber(prop, signal) => {
            #[cfg(target_arch = "wasm32")]
            {
                create_effect({
                    let signal = signal.clone();
                    let el = el.clone();
                    move || el.set_property(prop, &JsValue::from_f64(signal.get()))
                });
            }
            el.event(ev::input, {
                move |event: Event| {
                    signal.set(
                        JsValue::as_f64(
                            &Reflect::get(&event.target().unwrap(), &prop.into()).unwrap(),
                        )
                        .unwrap(),
                    );
                }
            });
        }
        AttributeValue::BindString(prop, signal) => {
            #[cfg(target_arch = "wasm32")]
            {
                create_effect({
                    let signal = signal.clone();
                    let el = el.clone();
                    move || el.set_property(prop, &JsValue::from_str(&signal.get_clone()))
                });
            }
            el.event(ev::input, {
                let signal = Clone::clone(&signal);
                move |event: Event| {
                    signal.set(
                        JsValue::as_string(
                            &Reflect::get(&event.target().unwrap(), &prop.into()).unwrap(),
                        )
                        .unwrap(),
                    );
                }
            });
        }
        AttributeValue::Property(prop, value) => {
            create_effect(move || el.set_property(prop, &value));
        }
        AttributeValue::Ref(value) => {
            value.set(el);
        }
    };
}

//! Abstractions for events and event handlers.

use std::future::Future;

use sycamore_reactive::Scope;

use crate::generic_node::GenericNode;

/// A trait that is implemented for all event handlers.
pub trait EventHandler<'a, T, Ev, S>
where
    Ev: EventDescriptor<T>,
{
    fn call(&mut self, cx: Scope<'a>, event: Ev::EventData);
}

impl<'a, T, Ev, F> EventHandler<'a, T, Ev, ()> for F
where
    Ev: EventDescriptor<T>,
    F: FnMut(Ev::EventData),
{
    fn call(&mut self, _cx: Scope<'a>, event: Ev::EventData) {
        self(event)
    }
}

#[cfg(feature = "suspense")]
impl<'a, G, Ev, F, Fut> EventHandler<'a, G, Ev, ((),)> for F
where
    G: GenericNode,
    Ev: EventDescriptor<G>,
    F: FnMut(Ev::EventData) -> Fut,
    Fut: Future<Output = ()> + 'a,
{
    fn call(&mut self, cx: Scope<'a>, event: Ev::EventData) {
        sycamore_futures::spawn_local_scoped(cx, self(event));
    }
}

/// An event descriptor describing event data which can be converted between `T` and itself.
pub trait EventDescriptor<T>: 'static {
    /// The type of the event data that is passed to the event handler.
    type EventData: From<T> + Into<T>;
    /// The name of the event.
    const EVENT_NAME: &'static str;
}

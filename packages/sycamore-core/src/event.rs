//! Abstractions for events and event handlers.

#[cfg(feature = "suspense")]
use std::future::Future;

/// A trait that is implemented for all event handlers.
pub trait EventHandler<T, Ev, S>
where
    Ev: EventDescriptor<T>,
{
    fn call(&mut self, event: Ev::EventData);
}

impl<T, Ev, F> EventHandler<T, Ev, ()> for F
where
    Ev: EventDescriptor<T>,
    F: FnMut(Ev::EventData),
{
    fn call(&mut self, event: Ev::EventData) {
        self(event)
    }
}

#[cfg(feature = "suspense")]
impl<T, Ev, F, Fut> EventHandler<T, Ev, ((), ())> for F
where
    Ev: EventDescriptor<T>,
    F: FnMut(Ev::EventData) -> Fut,
    Fut: Future<Output = ()> + 'static,
{
    fn call(&mut self, event: Ev::EventData) {
        sycamore_futures::spawn_local_scoped(self(event));
    }
}

/// An event descriptor describing event data which can be converted between `T` and itself.
pub trait EventDescriptor<T>: 'static {
    /// The type of the event data that is passed to the event handler.
    type EventData: From<T> + Into<T>;

    /// The name of the event.
    const EVENT_NAME: &'static str;
}

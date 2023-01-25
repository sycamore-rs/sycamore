//! Definitions for events that can be used with the [`on`] directive.

#![allow(non_camel_case_types)]

use sycamore_core2::generic_node::GenericNodeElements;
use sycamore_reactive::Scope;
use wasm_bindgen::JsValue;
use web_sys::*;

use super::{Attributes, WebElement};
use crate::ElementBuilder;

pub trait OnAttributes<'a> {
    fn on<T: From<JsValue> + 'a, S>(
        self,
        event: OnAttr<T>,
        handler: impl EventHandler<'a, T, S> + 'a,
    ) -> Self;
}

impl<'a, E: WebElement> OnAttributes<'a> for ElementBuilder<'a, E> {
    fn on<T: From<JsValue> + 'a, S>(
        mut self,
        event: OnAttr<T>,
        mut handler: impl EventHandler<'a, T, S> + 'a,
    ) -> Self {
        let cx = self.cx();
        self.mark_dyn();
        let type_erased = Box::new(move |ev: JsValue| handler.call(cx, ev.into()));
        self.as_node()
            .add_event_listener(cx, event.name, type_erased);
        self
    }
}
impl<'a, E: WebElement> OnAttributes<'a> for Attributes<'a, E> {
    fn on<T: From<JsValue> + 'a, S>(
        self,
        event: OnAttr<T>,
        handler: impl EventHandler<'a, T, S> + 'a,
    ) -> Self {
        self.add_fn(|builder| {
            builder.on(event, handler);
        });
        self
    }
}

/// Attribute directive for attaching an event listener to an element.
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

/// Macro for defining event types.
macro_rules! define_events {
    (
        $(
            $(#[$attr:meta])*
            $ev:ident : $ev_ty:ty,
        )*
    ) => {
        #[allow(non_upper_case_globals)]
        impl on {
            $(
                #[doc = concat!("The ", stringify!($ev), " event.")]
                $(#[$attr])*
                pub const $ev: OnAttr<$ev_ty> = OnAttr::new(stringify!($ev));
            )*
        }
    };
}

define_events! {
    /*
    WindowEventHandlersEventMap
    */
    afterprint: Event,
    beforeprint: Event,
    beforeunload: BeforeUnloadEvent,
    gamepadconnected: GamepadEvent,
    gamepaddisconnected: GamepadEvent,
    hashchange: HashChangeEvent,
    languagechange: Event,
    message: MessageEvent,
    messageerror: MessageEvent,
    offline: Event,
    online: Event,
    pagehide: PageTransitionEvent,
    pageshow: PageTransitionEvent,
    popstate: PopStateEvent,
    rejectionhandled: PromiseRejectionEvent,
    storage: StorageEvent,
    unhandledrejection: PromiseRejectionEvent,
    unload: Event,

    /*
    GlobalEventHandlersEventMap
    */
    abort: UiEvent,
    animationcancel: AnimationEvent,
    animationend: AnimationEvent,
    animationiteration: AnimationEvent,
    animationstart: AnimationEvent,
    auxclick: MouseEvent,
    beforeinput: InputEvent,
    blur: FocusEvent,
    canplay: Event,
    canplaythrough: Event,
    change: Event,
    click: MouseEvent,
    close: Event,
    compositionend: CompositionEvent,
    compositionstart: CompositionEvent,
    compositionupdate: CompositionEvent,
    contextmenu: MouseEvent,
    cuechange: Event,
    dblclick: MouseEvent,
    drag: DragEvent,
    dragend: DragEvent,
    dragenter: DragEvent,
    dragleave: DragEvent,
    dragover: DragEvent,
    dragstart: DragEvent,
    drop: DragEvent,
    durationchange: Event,
    emptied: Event,
    ended: Event,
    error: ErrorEvent,
    focus: FocusEvent,
    focusin: FocusEvent,
    focusout: FocusEvent,
    formdata: Event, // web_sys does not include `FormDataEvent`
    gotpointercapture: PointerEvent,
    input: Event,
    invalid: Event,
    keydown: KeyboardEvent,
    keypress: KeyboardEvent,
    keyup: KeyboardEvent,
    load: Event,
    loadeddata: Event,
    loadedmetadata: Event,
    loadstart: Event,
    lostpointercapture: PointerEvent,
    mousedown: MouseEvent,
    mouseenter: MouseEvent,
    mouseleave: MouseEvent,
    mousemove: MouseEvent,
    mouseout: MouseEvent,
    mouseover: MouseEvent,
    mouseup: MouseEvent,
    pause: Event,
    play: Event,
    playing: Event,
    pointercancel: PointerEvent,
    pointerdown: PointerEvent,
    pointerenter: PointerEvent,
    pointerleave: PointerEvent,
    pointermove: PointerEvent,
    pointerout: PointerEvent,
    pointerover: PointerEvent,
    pointerup: PointerEvent,
    progress: ProgressEvent,
    ratechange: Event,
    reset: Event,
    resize: UiEvent,
    scroll: Event,
    securitypolicyviolation: SecurityPolicyViolationEvent,
    seeked: Event,
    seeking: Event,
    select: Event,
    selectionchange: Event,
    selectstart: Event,
    slotchange: Event,
    stalled: Event,
    submit: SubmitEvent,
    suspend: Event,
    timeupdate: Event,
    toggle: Event,
    touchcancel: TouchEvent,
    touchend: TouchEvent,
    touchmove: TouchEvent,
    touchstart: TouchEvent,
    transitioncancel: TransitionEvent,
    transitionend: TransitionEvent,
    transitionrun: TransitionEvent,
    transitionstart: TransitionEvent,
    volumechange: Event,
    waiting: Event,
    webkitanimationend: Event,
    webkitanimationiteration: Event,
    webkitanimationstart: Event,
    webkittransitionend: Event,
    wheel: WheelEvent,

    /*
    WindowEventMap
    */
    DOMContentLoaded: Event,
    devicemotion: DeviceMotionEvent,
    deviceorientation: DeviceOrientationEvent,
    orientationchange: Event,

    /*
    DocumentAndElementEventHandlersEventMap
    */
    copy: Event, // ClipboardEvent is unstable
    cut: Event, // ClipboardEvent is unstable
    paste: Event, // ClipboardEvent is unstable

    /*
    DocumentEventMap
    */
    fullscreenchange: Event,
    fullscreenerror: Event,
    pointerlockchange: Event,
    pointerlockerror: Event,
    readystatechange: Event,
    visibilitychange: Event,
}

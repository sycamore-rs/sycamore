//! Definition for all the events that can be listened to.

use wasm_bindgen::JsCast;
use web_sys::{
    AnimationEvent, BeforeUnloadEvent, CompositionEvent, DeviceMotionEvent, DeviceOrientationEvent,
    DragEvent, ErrorEvent, Event, FocusEvent, GamepadEvent, HashChangeEvent, InputEvent,
    KeyboardEvent, MessageEvent, MouseEvent, PageTransitionEvent, PointerEvent, PopStateEvent,
    ProgressEvent, PromiseRejectionEvent, SecurityPolicyViolationEvent, StorageEvent, SubmitEvent,
    TouchEvent, TransitionEvent, UiEvent, WheelEvent,
};

/// Description of a type of event.
pub trait EventDescriptor {
    /// The JS type of the event data that is passed to the event handler.
    type EventTy: Into<Event> + JsCast;
    /// The name of the event.
    const NAME: &'static str;
}

macro_rules! impl_event {
    ($name:ident: $ty:ty) => {
        #[allow(non_camel_case_types)]
        pub struct $name;
        impl EventDescriptor for $name {
            type EventTy = $ty;
            const NAME: &'static str = stringify!($name);
        }
    };
}

macro_rules! impl_events {
    ($($name:ident: $ty:ty,)*) => {
        $(impl_event!($name: $ty);)*
    };
}

impl_events! {
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

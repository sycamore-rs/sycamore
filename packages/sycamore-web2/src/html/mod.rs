//! HTML and SVG tag definitions.
//!
//! _Documentation sources: <https://developer.mozilla.org/en-US/>_

use sycamore_reactive::create_scope_immediate;

pub mod attr;
pub mod elements;
pub mod events;
pub mod props;

pub fn test() {
    create_scope_immediate(|cx| {
        use attr::attr;
        use events::on;

        let node = elements::button::new(cx)
            .with(attr::class, "bg-red-500")
            .with(on::click, |_| {})
            .into_element();
        let view = crate::View::new_node(node);

        let _ = view;
    });
}

/// Macro for defining event types.
macro_rules! define_events {
    (
        $(
            $(#[$attr:meta])*
            $ev:ident : $ev_ty:ty,
        )*
    ) => {
        $(
            #[doc = concat!("The ", stringify!($ev), " event.")]
            #[doc = stringify!($event)]
            $(#[$attr])*
            #[allow(non_camel_case_types)]
            #[allow(missing_docs)]
            #[derive(Debug)]
            pub struct $ev;
            // impl EventDescriptor<JsValue> for $ev {
            //     type EventData = $ev_ty;
            //     const EVENT_NAME: &'static str = stringify!($ev);
            // }
        )*
    };
}

/// HTML events definitions.
pub mod ev {
    // use sycamore_core2::web::html::EventDescriptor;
    // use wasm_bindgen::JsValue;
    // use web_sys::*;

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
}

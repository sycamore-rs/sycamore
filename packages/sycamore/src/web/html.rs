//! HTML and SVG tag definitions.
//!
//! _Documentation sources: <https://developer.mozilla.org/en-US/>_

pub use sycamore_core::event::{EventDescriptor, EventHandler};
pub use sycamore_web::on_mount;

#[allow(clippy::useless_attribute)] // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/10878
#[allow(ambiguous_glob_reexports)] // FIXME: fix this lint warning
pub use self::html_tags::*;
pub use self::svg_tags::*;
use crate::builder::ElementBuilder;
use crate::generic_node::{GenericNodeElements, SycamoreElement};

/// Macro for generating element definitions.
macro_rules! define_elements {
    (
        $ns:expr,
        $doc:expr,
        $(
            $(#[$attr:meta])*
            $el:ident $(($tag:expr))? {
                $(
                    $(#[$attr_method:meta])*
                    $at:ident: $ty:path,
                )*
            },
        )*
    ) => {
        $(
            define_element_impl! {
                $ns,
                $doc,
                $($tag)*,
                $(#[$attr])*
                $el {
                    $(
                        $(#[$attr_method])*
                        $at: $ty
                    )*
                }
            }
        )*
        /// Builder methods for elements.
        pub mod builder {
            use super::*;

            $(
                define_element_builder_impl! {
                    $ns,
                    $doc,
                    $($tag)*,
                    $(#[$attr])*
                    $el {
                        $(
                            $(#[$attr_method])*
                            $at: $ty
                        )*
                    }
                }
            )*
        }
    };
}

macro_rules! define_element_impl {
    (
        $ns:expr,
        $tag:expr,
        $(#[$attr:meta])*
        $el:ident {
            $(
                $(#[$attr_method:meta])*
                $at:ident: $ty:path,
            )*
        }
    ) => {
        #[allow(non_camel_case_types)]
        $(#[$attr])*
        #[derive(Debug)]
        pub struct $el {}

        impl SycamoreElement for $el {
            const TAG_NAME: &'static str = $tag;
            const NAMESPACE: Option<&'static str> = $ns;
        }
    };
    (
        $ns:expr,
        $doc:expr,
        $tag:expr,
        $(#[$attr:meta])*
        $el:ident $rest:tt
    ) => {
        define_element_impl! {
            $ns,
            $tag,
            #[doc = concat!("Build a [`<", $tag, ">`](", $doc, $tag, ") element.")]
            $(#[$attr])*
            $el $rest
        }
    };
    (
        $ns:expr,
        $doc:expr,,
        $(#[$attr:meta])*
        $el:ident $rest:tt
    ) => {
        define_element_impl! {
            $ns,
            $doc,
            stringify!($el),
            $(#[$attr])*
            $el $rest
        }
    };
}

macro_rules! define_element_builder_impl {
    (
        $ns:expr,
        $tag:expr,
        $(#[$attr:meta])*
        $el:ident {
            $(
                $(#[$attr_method:meta])*
                $at:ident: $ty:path,
            )*
        }
    ) => {
        #[allow(non_snake_case)]
        $(#[$attr])*
        pub fn $el<G: GenericNodeElements>() -> ElementBuilder<G, impl FnOnce() -> G> {
            ElementBuilder::new(move || G::element::<$el>())
        }
    };
    (
        $ns:expr,
        $doc:expr,
        $tag:expr,
        $(#[$attr:meta])*
        $el:ident $rest:tt
    ) => {
        define_element_builder_impl! {
            $ns,
            $tag,
            #[doc = concat!("Build a [`<", $tag, ">`](", $doc, $tag, ") element.")]
            $(#[$attr])*
            $el $rest
        }
    };
    (
        $ns:expr,
        $doc:expr,,
        $(#[$attr:meta])*
        $el:ident $rest:tt
    ) => {
        define_element_builder_impl! {
            $ns,
            $doc,
            stringify!($el),
            $(#[$attr])*
            $el $rest
        }
    };
}

/// HTML tag definitions.
#[doc(hidden)]
pub mod html_tags {
    use super::*;
    // A list of valid HTML5 elements (does not include removed or obsolete elements).
    define_elements! {
        None,
        "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/",
        /// The `<a>` HTML element (or anchor element), with its `href` attribute, creates a hyperlink to web pages, files, email addresses, locations in the same page, or anything else a URL can address.
        ///
        /// Content within each `<a>` should indicate the link's destination. If the `href` attribute is present, pressing the enter key while focused on the `<a>` element will activate it.
        a {},
        abbr {},
        address {},
        area {},
        article {},
        aside {},
        audio {},
        b {},
        base {},
        bdi {},
        bdo {},
        blockquote {},
        body {},
        br {},
        /// The `<button>` HTML element represents a clickable button, used to submit forms or anywhere in a document for accessible, standard button functionality.
        ///
        /// By default, HTML buttons are presented in a style resembling the platform the user agent runs on, but you can change buttons’ appearance with CSS.
        button {},
        canvas {},
        caption {},
        cite {},
        code {},
        col {},
        colgroup {},
        data {},
        datalist {},
        dd {},
        del {},
        details {},
        dfn {},
        dialog {},
        /// The `<div>` HTML element is the generic container for flow content. It has no effect on the content or layout until styled in some way using CSS (e.g. styling is directly applied to it, or some kind of layout model like Flexbox is applied to its parent element).
        ///
        /// As a "pure" container, the `<div>` element does not inherently represent anything. Instead, it's used to group content so it can be easily styled using the class or id attributes, marking a section of a document as being written in a different language (using the lang attribute), and so on.
        ///
        /// # Usage notes
        /// The `<div>` element should be used only when no other semantic element (such as `<article>` or `<nav>`) is appropriate.
        div {},
        dl {},
        dt {},
        em {},
        embed {},
        fieldset {},
        figcaption {},
        figure {},
        footer {},
        form {},
        head {},
        header {},
        hgroup {},
        h1 {},
        h2 {},
        h3 {},
        h4 {},
        h5 {},
        h6 {},
        hr {},
        html {},
        i {},
        iframe {},
        img {},
        /// The `<input>` HTML element is used to create interactive controls for web-based forms in order to accept data from the user; a wide variety of types of input data and control widgets are available, depending on the device and user agent. The `<input>` element is one of the most powerful and complex in all of HTML due to the sheer number of combinations of input types and attributes.
        input {},
        ins {},
        kbd {},
        keygen {},
        /// The `<label>` HTML element represents a caption for an item in a user interface.
        ///
        /// Associating a `<label>` with an `<input>` element offers some major advantages:
        /// * The label text is not only visually associated with its corresponding text input; it is programmatically associated with it too. This means that, for example, a screen reader will read out the label when the user is focused on the form input, making it easier for an assistive technology user to understand what data should be entered.
        /// * When a user clicks or touches/taps a label, the browser passes the focus to its associated input (the resulting event is also raised for the input). That increased hit area for focusing the input provides an advantage to anyone trying to activate it — including those using a touch-screen device.
        ///
        /// To associate the `<label>` with an `<input>` element, you need to give the `<input>` an `id` attribute. The `<label>` then needs a for attribute whose value is the same as the input's `id`.
        ///
        /// Alternatively, you can nest the `<input>` directly inside the `<label>`, in which case the `for` and `id` attributes are not needed because the association is implicit:
        ///
        /// ```html
        /// <label>Do you like peas?
        ///   <input type="checkbox" name="peas">
        /// </label>
        /// ```
        /// The form control that a label is labeling is called the labeled control of the label element. Multiple labels can be associated with the same form control:
        ///
        /// ```html
        /// <label for="username">Enter your username:</label>
        /// <input id="username">
        /// <label for="username">Forgot your username?</label>
        /// ```
        /// Elements that can be associated with a `<label>` element include `<button>`, `<input>` (except for `type="hidden"`), `<meter>`, `<output>`, `<progress>`, `<select>` and `<textarea>`.
        label {},
        legend {},
        /// The `<li>` HTML element is used to represent an item in a list. It must be contained in a parent element: an ordered list (`<ol>`), an unordered list (`<ul>`), or a menu (`<menu>`). In menus and unordered lists, list items are usually displayed using bullet points. In ordered lists, they are usually displayed with an ascending counter on the left, such as a number or letter.
        li {},
        link {},
        main {},
        map {},
        mark {},
        menu {},
        menuitem {},
        meta {},
        meter {},
        nav {},
        noscript {},
        object {},
        /// The `<ol>` HTML element represents an ordered list of items — typically rendered as a numbered list.
        ol {},
        optgroup {},
        option {},
        output {},
        /// The `<p>` HTML element represents a paragraph. Paragraphs are usually represented in visual media as blocks of text separated from adjacent blocks by blank lines and/or first-line indentation, but HTML paragraphs can be any structural grouping of related content, such as images or form fields.
        ///
        /// Paragraphs are block-level elements, and notably will automatically close if another block-level element is parsed before the closing `</p>` tag.
        p {},
        param {},
        picture {},
        pre {},
        progress {},
        q {},
        rp {},
        rt {},
        ruby {},
        s {},
        samp {},
        script {},
        section {},
        select {},
        small {},
        source {},
        /// The `<span>` HTML element is a generic inline container for phrasing content, which does not inherently represent anything. It can be used to group elements for styling purposes (using the class or id attributes), or because they share attribute values, such as lang. It should be used only when no other semantic element is appropriate. `<span>` is very much like a `<div>` element, but `<div>` is a block-level element whereas a `<span>` is an inline element.
        span {},
        strong {},
        style {},
        sub {},
        summary {},
        sup {},
        table {},
        tbody {},
        td {},
        template {},
        textarea {},
        tfoot {},
        th {},
        thead {},
        time {},
        title {},
        tr {},
        track {},
        u {},
        /// The `<ul>` HTML element represents an unordered list of items, typically rendered as a bulleted list.
        ul {},
        var {},
        video {},
        wbr {},
    }
}

/// SVG tag definitions
#[doc(hidden)]
pub mod svg_tags {
    use super::*;
    // A list of valid SVG elements. Some elements are commented out because they conflict with the
    // HTML elements.
    define_elements! {
        Some("http://www.w3.org/2000/svg"),
        "https://developer.mozilla.org/en-US/docs/Web/SVG/Element/",
        svg {},
        svg_a("a") {},
        animate {},
        animateMotion {},
        animateTransform {},
        circle {},
        clipPath {},
        defs {},
        desc {},
        discard {},
        ellipse {},
        feBlend {},
        feColorMatrix {},
        feComponentTransfer {},
        feComposite {},
        feConvolveMatrix {},
        feDiffuseLighting {},
        feDisplacementMap {},
        feDistantLight {},
        feDropShadow {},
        feFlood {},
        feFuncA {},
        feFuncB {},
        feFuncG {},
        feFuncR {},
        feGaussianBlur {},
        feImage {},
        feMerge {},
        feMergeNode {},
        feMorphology {},
        feOffset {},
        fePointLight {},
        feSpecularLighting {},
        feSpotLight {},
        feTile {},
        feTurbulence {},
        filter {},
        foreignObject {},
        g {},
        hatch {},
        hatchpath {},
        image {},
        line {},
        linearGradient {},
        marker {},
        mask {},
        metadata {},
        mpath {},
        path {},
        pattern {},
        polygon {},
        polyline {},
        radialGradient {},
        rect {},
        svg_script("script") {},
        set {},
        stop {},
        svg_style("style") {},
        switch {},
        symbol {},
        text {},
        textPath {},
        svg_title("title") {},
        tspan {},
        r#use("use") {},
        view {},
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
        $(
            #[doc = concat!("The ", stringify!($ev), " event.")]
            #[doc = stringify!($event)]
            $(#[$attr])*
            #[allow(non_camel_case_types)]
            #[allow(missing_docs)]
            #[derive(Debug)]
            pub struct $ev;
            impl EventDescriptor<JsValue> for $ev {
                type EventData = $ev_ty;
                const EVENT_NAME: &'static str = stringify!($ev);
            }
        )*
    };
}

/// HTML events definitions.
pub mod ev {
    use wasm_bindgen::JsValue;
    use web_sys::*;

    use crate::web::html::EventDescriptor;

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

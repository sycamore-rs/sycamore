#![allow(non_snake_case)]

use events::EventHandler;

use crate::*;

/// Create an HTML element with `tag`.
// TODO: deprecate
pub(crate) fn element(tag: &'static str) -> HtmlNode {
    HtmlNode::create_element(tag.into())
}

/// Create a SVG element with `tag`.
pub(crate) fn svg_element(tag: &'static str) -> HtmlNode {
    HtmlNode::create_element_ns("http://www.w3.org/2000/svg", tag.into())
}

/// A struct representing a custom element. This can be created by calling [`custom_element`].
pub struct CustomElement(HtmlNode);

/// Create a new custom element with `tag`.
pub fn custom_element(tag: &'static str) -> CustomElement {
    CustomElement(element(tag))
}

impl From<CustomElement> for View {
    fn from(el: CustomElement) -> Self {
        View::from_node(el.0)
    }
}

impl IntoHtmlNode for CustomElement {
    fn into_html_node(self) -> HtmlNode {
        self.0
    }
    fn as_html_node(&self) -> &HtmlNode {
        &self.0
    }
    fn as_html_node_mut(&mut self) -> &mut HtmlNode {
        &mut self.0
    }
}

impl GlobalAttributes for CustomElement {}
impl HtmlGlobalAttributes for CustomElement {}

macro_rules! impl_attribute {
    ($(#[$attr:meta])* $v:vis $ident:ident: $ty:ty) => {
        impl_attribute!($(#[$attr])* $v $ident (stringify!($ident)): $ty);
    };
    ($(#[$attr:meta])* $v:vis $ident:ident ($name:expr): $ty:ty) => {
        $(#[$attr])*
        $v fn $ident(mut self, value: impl Into<$ty>) -> Self {
            set_attribute(self.as_html_node_mut(), $name, value.into());
            self
        }
    }
}

macro_rules! impl_attributes {
    ($(
        $(#[$attr:meta])*
        $v:vis $ident:ident $(($name:literal))?: $ty:ty,
    )*) => {
        $(
            impl_attribute!($(#[$attr])* $v $ident $(($name))*: $ty);
        )*
    };
}

macro_rules! impl_element {
    (
        $(#[$attr:meta])*
        $name:ident {
            $(
                $(#[$prop_attr:meta])*
                $prop:ident $(($prop_name:literal))?: $ty:ty,
            )*
        }
    ) => {
        impl_element!($(#[$attr])* $name (stringify!($name)) {
            $(
                $(#[$prop_attr])*
                $prop $(($prop_name))*: $ty,
            )*
        });
    };
    (
        $(#[$attr:meta])*
        $name:ident ($tag:expr) {
            $(
                $(#[$prop_attr:meta])*
                $prop:ident $(($prop_name:literal))?: $ty:ty,
            )*
        }
    ) => {
        paste::paste! {
            #[doc = "The `<" $name ">` HTML element. This can be created by calling [`" $name "()`]."]
            pub struct [<Html $name:camel>] (HtmlNode);

            #[doc = "Create a `<" $name ">` element."]
            #[doc = ""]
            $(#[$attr])*
            pub fn $name() -> [<Html $name:camel>] {
                [<Html $name:camel>](element($tag))
            }

            impl From<[<Html $name:camel>]> for View {
                fn from(el: [<Html $name:camel>]) -> Self {
                    View::from_node(el.0)
                }
            }

            impl IntoHtmlNode for [<Html $name:camel>] {
                fn into_html_node(self) -> HtmlNode {
                    self.0
                }
                fn as_html_node(&self) -> &HtmlNode {
                    &self.0
                }
                fn as_html_node_mut(&mut self) -> &mut HtmlNode {
                    &mut self.0
                }
            }

            impl GlobalAttributes for [<Html $name:camel>] {}
            impl HtmlGlobalAttributes for [<Html $name:camel>] {}

            pub trait [<Html $name:camel Attributes>]: IntoHtmlNode + Sized {
                impl_attributes! {
                    $(
                        $(#[$prop_attr])*
                        $prop $(($prop_name))*: $ty,
                    )*
                }
            }

            impl [<Html $name:camel Attributes>] for [<Html $name:camel>] {}
        }
    };
}

macro_rules! impl_elements {
    ($(
        $(#[$attr:meta])*
        $name:ident $(($tag:expr))? {
            $(
                $(#[$prop_attr:meta])*
                $prop:ident $(($prop_name:literal))?: $ty:ty,
            )*
        },
    )*) => {
        $(
            impl_element!($(#[$attr])* $name $(($tag))* {
                $(
                    $(#[$prop_attr])*
                    $prop $(($prop_name))*: $ty,
                )*
            });
        )*
        /// Module that includes all the HTML attribute traits and is intended to be glob exported.
        pub mod html_attributes {
            paste::paste! {
                pub use super::{$([<Html $name:camel Attributes>]),*};
            }
        }
    };
}

macro_rules! impl_svg_element {
    (
        $(#[$attr:meta])*
        $name:ident {
            $(
                $(#[$prop_attr:meta])*
                $prop:ident $(($prop_name:literal))?: $ty:ty,
            )*
        }
    ) => {
        impl_svg_element!($(#[$attr])* $name (stringify!($name)) {
            $(
                $(#[$prop_attr])*
                $prop $(($prop_name))*: $ty,
            )*
        });
    };
    (
        $(#[$attr:meta])*
        $name:ident ($tag:expr) {
            $(
                $(#[$prop_attr:meta])*
                $prop:ident $(($prop_name:literal))?: $ty:ty,
            )*
        }
    ) => {
        paste::paste! {
            #[doc = "The `<" $name ">` SVG element. This can be created by calling [`" $name "()`]."]
            pub struct [<Svg $name:camel>] (HtmlNode);

            #[doc = "Create a `<" $name ">` element."]
            #[doc = ""]
            $(#[$attr])*
            pub fn $name() -> [<Svg $name:camel>] {
                [<Svg $name:camel>](svg_element($tag))
            }

            impl From<[<Svg $name:camel>]> for View {
                fn from(el: [<Svg $name:camel>]) -> Self {
                    View::from_node(el.0)
                }
            }

            impl IntoHtmlNode for [<Svg $name:camel>] {
                fn into_html_node(self) -> HtmlNode {
                    self.0
                }
                fn as_html_node(&self) -> &HtmlNode {
                    &self.0
                }
                fn as_html_node_mut(&mut self) -> &mut HtmlNode {
                    &mut self.0
                }
            }

            impl GlobalAttributes for [<Svg $name:camel>] {}
            impl SvgGlobalAttributes for [<Svg $name:camel>] {}

            pub trait [<Svg $name:camel Attributes>]: IntoHtmlNode + Sized {
                impl_attributes! {
                    $(
                        $(#[$prop_attr])*
                        $prop $(($prop_name))*: $ty,
                    )*
                }
            }

            impl [<Svg $name:camel Attributes>] for [<Svg $name:camel>] {}
        }
    };
}

macro_rules! impl_svg_elements {
    ($(
        $(#[$attr:meta])*
        $name:ident $(($tag:expr))? {
            $(
                $(#[$prop_attr:meta])*
                $prop:ident $(($prop_name:literal))?: $ty:ty,
            )*
        },
    )*) => {
        $(
            impl_svg_element!($(#[$attr])* $name $(($tag))* {
                $(
                    $(#[$prop_attr])*
                    $prop $(($prop_name))*: $ty,
                )*
            });
        )*
        /// Module that includes all the Svg attribute traits and is intended to be glob exported.
        pub mod svg_attributes {
            paste::paste! {
                pub use super::{$([<Svg $name:camel Attributes>]),*};
            }
        }
    };
}

/// Definition of all the HTML and SVG elements.
pub mod tags {
    use super::*;

    impl_elements! {
        /// The `<a>` HTML element (or anchor element), with its `href` attribute, creates a hyperlink to web pages, files, email addresses, locations in the same page, or anything else a URL can address.
        ///
        /// Content within each `<a>` should indicate the link's destination. If the `href` attribute is present, pressing the enter key while focused on the `<a>` element will activate it.
        a {
            download: MaybeDynString,
            href: MaybeDynString,
            hreflang: MaybeDynString,
            target: MaybeDynString,
            r#type("type"): MaybeDynString,
            ping: MaybeDynString,
            rel: MaybeDynString,
        },
        abbr {},
        address {},
        area {
            alt: MaybeDynString,
            coords: MaybeDynString,
            download: MaybeDynString,
            href: MaybeDynString,
            hreflang: MaybeDynString,
            media: MaybeDynString,
            referrerpolicy: MaybeDynString,
            ping: MaybeDynString,
            rel: MaybeDynString,
            shape: MaybeDynString,
            target: MaybeDynString,
            r#type("type"): MaybeDynString,
        },
        article {},
        aside {},
        audio {
            autoplay: MaybeDynBool,
            controls: MaybeDynBool,
            crossorigin: MaybeDynString,
            muted: MaybeDynBool,
            preload: MaybeDynString,
            src: MaybeDynString,
            r#loop("loop"): MaybeDynBool,
        },
        b {},
        base {
            href: MaybeDynString,
            target: MaybeDynString,
        },
        bdi {},
        bdo {},
        blockquote {
            cite: MaybeDynString,
        },
        body {},
        br {},
        /// The `<button>` HTML element represents a clickable button, used to submit forms or anywhere in a document for accessible, standard button functionality.
        ///
        /// By default, HTML buttons are presented in a style resembling the platform the user agent runs on, but you can change buttons’ appearance with CSS.
        button {
            autofocus: MaybeDynBool,
            disabled: MaybeDynBool,
            form: MaybeDynString,
            formaction: MaybeDynString,
            formenctype: MaybeDynString,
            formmethod: MaybeDynString,
            formnovalidate: MaybeDynBool,
            formtarget: MaybeDynString,
            name: MaybeDynString,
            r#type("type"): MaybeDynString,
            value: MaybeDynString,
        },
        canvas {
            height: MaybeDynString, // TODO: int value
            width: MaybeDynString, // TODO: int value
        },
        caption {},
        cite {},
        code {
            language: MaybeDynString,
        },
        col {
            span: MaybeDynString, // TODO: int value
        },
        colgroup {
            span: MaybeDynString, // TODO: int value
        },
        data {
            value: MaybeDynString,
        },
        datalist {},
        dd {},
        del {
            cite: MaybeDynString,
            datetime: MaybeDynString,
        },
        details {
            open: MaybeDynBool,
        },
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
        embed {
            height: MaybeDynString,
            src: MaybeDynString,
            r#type("type"): MaybeDynString,
            width: MaybeDynString, // TODO: int value
        },
        fieldset {},
        figcaption {},
        figure {},
        footer {},
        form {
            acceptcharset: MaybeDynString,
            action: MaybeDynString,
            autocomplete: MaybeDynString,
            enctype: MaybeDynString,
            method: MaybeDynString,
            name: MaybeDynString,
            novalidate: MaybeDynBool,
            target: MaybeDynString,
        },
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
        iframe {
            allow: MaybeDynString,
            allowfullscreen: MaybeDynBool,
            allowpaymentrequest: MaybeDynBool,
            height: MaybeDynString,
            loading: MaybeDynString,
            name: MaybeDynString,
            referrerpolicy: MaybeDynString,
            sandbox: MaybeDynBool,
            src: MaybeDynString,
            srcdoc: MaybeDynString,
            width: MaybeDynString,
        },
        img {
            alt: MaybeDynString,
            crossorigin: MaybeDynString,
            decoding: MaybeDynString,
            height: MaybeDynString,
            ismap: MaybeDynBool,
            loading: MaybeDynString,
            referrerpolicy: MaybeDynString,
            sizes: MaybeDynString,
            src: MaybeDynString,
            srcset: MaybeDynString,
            usemap: MaybeDynString,
            width: MaybeDynString,
        },
        /// The `<input>` HTML element is used to create interactive controls for web-based forms in order to accept data from the user; a wide variety of types of input data and control widgets are available, depending on the device and user agent. The `<input>` element is one of the most powerful and complex in all of HTML due to the sheer number of combinations of input types and attributes.
        input {
            accept: MaybeDynString,
            alt: MaybeDynString,
            autocomplete: MaybeDynString,
            autofocus: MaybeDynBool,
            capture: MaybeDynString,
            checked: MaybeDynBool,
            directory: MaybeDynString,
            disabled: MaybeDynBool,
            form: MaybeDynString,
            formaction: MaybeDynString,
            formenctype: MaybeDynString,
            formmethod: MaybeDynString,
            formnovalidate: MaybeDynBool,
            formtarget: MaybeDynString,
            height: MaybeDynString, // TODO: int value
            initial_checked: MaybeDynBool,
            initial_value: MaybeDynString,
            list: MaybeDynString,
            max: MaybeDynString,
            maxlength: MaybeDynString, // TODO: int value
            min: MaybeDynString,
            minlength: MaybeDynString, // TODO: int value
            multiple: MaybeDynBool,
            name: MaybeDynString,
            pattern: MaybeDynString,
            placeholder: MaybeDynString,
            readonly: MaybeDynBool,
            required: MaybeDynBool,
            size: MaybeDynString, // TODO: int value
            spellcheck: MaybeDynBool,
            src: MaybeDynString,
            step: MaybeDynString,
            tabindex: MaybeDynString, // TODO: int value
            r#type("type"): MaybeDynString,
            value: MaybeDynString,
            width: MaybeDynString, // TODO: int value
        },
        ins {
            cite: MaybeDynString,
            datetime: MaybeDynString,
        },
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
        label {
            form: MaybeDynString,
            r#for("for"): MaybeDynString,
        },
        legend {},
        /// The `<li>` HTML element is used to represent an item in a list. It must be contained in a parent element: an ordered list (`<ol>`), an unordered list (`<ul>`), or a menu (`<menu>`). In menus and unordered lists, list items are usually displayed using bullet points. In ordered lists, they are usually displayed with an ascending counter on the left, such as a number or letter.
        li {
            value: MaybeDynString, // TODO: int value
        },
        link {
            crossorigin: MaybeDynString,
            href: MaybeDynString,
            hreflang: MaybeDynString,
            media: MaybeDynString,
            rel: MaybeDynString,
            sizes: MaybeDynString,
            title: MaybeDynString,
            r#type("type"): MaybeDynString,
            integrity: MaybeDynString,
        },
        main {},
        map {
            name: MaybeDynString,
        },
        mark {},
        menu {},
        menuitem {},
        meta {
            charset: MaybeDynString,
            content: MaybeDynString,
            http_equiv("http-equiv"): MaybeDynString,
            name: MaybeDynString,
        },
        meter {
            value: MaybeDynString, // TODO: int value
            min: MaybeDynString, // TODO: int value
            max: MaybeDynString, // TODO: int value
            low: MaybeDynString, // TODO: int value
            high: MaybeDynString, // TODO: int value
            optimum: MaybeDynString, // TODO: int value
            form: MaybeDynString,
        },
        nav {},
        noscript {},
        object {
            data: MaybeDynString,
            form: MaybeDynString,
            height: MaybeDynString, // TODO: int value
            name: MaybeDynString,
            r#type("type"): MaybeDynString,
            typemustmatch: MaybeDynBool,
            usemap: MaybeDynString,
            width: MaybeDynString,
        },
        /// The `<ol>` HTML element represents an ordered list of items — typically rendered as a numbered list.
        ol {
            reversed: MaybeDynBool,
            start: MaybeDynString, // TODO: int value
            r#type("type"): MaybeDynString,
        },
        optgroup {
            disabled: MaybeDynBool,
            label: MaybeDynString,
        },
        option {
            disabled: MaybeDynBool,
            initial_selected: MaybeDynBool,
            label: MaybeDynString,
            selected: MaybeDynBool,
            value: MaybeDynString,
        },
        output {
            r#for("for"): MaybeDynString,
            form: MaybeDynString,
            name: MaybeDynString,
        },
        /// The `<p>` HTML element represents a paragraph. Paragraphs are usually represented in visual media as blocks of text separated from adjacent blocks by blank lines and/or first-line indentation, but HTML paragraphs can be any structural grouping of related content, such as images or form fields.
        ///
        /// Paragraphs are block-level elements, and notably will automatically close if another block-level element is parsed before the closing `</p>` tag.
        p {},
        param {
            name: MaybeDynString,
            value: MaybeDynString,
        },
        picture {},
        pre {},
        progress {
            value: MaybeDynString, // TODO: f64 value
            max: MaybeDynString, // TODO: f64 value
        },
        q {
            cite: MaybeDynString,
        },
        rp {},
        rt {},
        ruby {},
        s {},
        samp {},
        script {
            r#async: MaybeDynBool,
            crossorigin: MaybeDynString,
            defer: MaybeDynBool,
            integrity: MaybeDynString,
            nomodule: MaybeDynBool,
            nonce: MaybeDynString,
            src: MaybeDynString,
            script: MaybeDynString,
            text: MaybeDynString,
            r#type("type"): MaybeDynString,
        },
        section {},
        select {
            autocomplete: MaybeDynString,
            autofocus: MaybeDynBool,
            disabled: MaybeDynBool,
            form: MaybeDynString,
            multiple: MaybeDynBool,
            name: MaybeDynString,
            required: MaybeDynBool,
            size: MaybeDynString, // TODO: int value
            value: MaybeDynString,
        },
        small {},
        source {
            src: MaybeDynString,
            r#type("type"): MaybeDynString,
        },
        /// The `<span>` HTML element is a generic inline container for phrasing content, which does not inherently represent anything. It can be used to group elements for styling purposes (using the class or id attributes), or because they share attribute values, such as lang. It should be used only when no other semantic element is appropriate. `<span>` is very much like a `<div>` element, but `<div>` is a block-level element whereas a `<span>` is an inline element.
        span {},
        strong {},
        style {
            media: MaybeDynString,
            nonce: MaybeDynString,
            title: MaybeDynString,
            r#type("type"): MaybeDynString,
        },
        sub {},
        summary {},
        sup {},
        table {},
        tbody {},
        td {
            colspan: MaybeDynString, // TODO: int value
            headers: MaybeDynString,
            rowspan: MaybeDynString, // TODO: int value
        },
        template {},
        textarea {
            autocomplete: MaybeDynString,
            autofocus: MaybeDynBool,
            cols: MaybeDynString, // TODO: int value
            disabled: MaybeDynBool,
            form: MaybeDynString,
            initial_value: MaybeDynString,
            maxlength: MaybeDynString, // TODO: int value
            minlength: MaybeDynString, // TODO: int value
            name: MaybeDynString,
            placeholder: MaybeDynString,
            readonly: MaybeDynBool,
            required: MaybeDynBool,
            rows: MaybeDynString, // TODO: int value
            spellcheck: MaybeDynBool,
            r#type("type"): MaybeDynString,
            value: MaybeDynString,
            wrap: MaybeDynString,
        },
        tfoot {},
        th {
            abbr: MaybeDynString,
            colspan: MaybeDynString, // TODO: int value
            headers: MaybeDynString,
            rowspan: MaybeDynString, // TODO: int value
            scope: MaybeDynString,
        },
        thead {},
        time {
            datetime: MaybeDynString,
        },
        title {},
        tr {},
        track {
            default: MaybeDynBool,
            kind: MaybeDynString,
            label: MaybeDynString,
            src: MaybeDynString,
            srclang: MaybeDynString,
        },
        u {},
        /// The `<ul>` HTML element represents an unordered list of items, typically rendered as a bulleted list.
        ul {},
        var {},
        video {
            autoplay: MaybeDynBool,
            controls: MaybeDynBool,
            crossorigin: MaybeDynString,
            height: MaybeDynString, // TODO: int value
            r#loop("loop"): MaybeDynBool,
            muted: MaybeDynBool,
            playsinline: MaybeDynBool,
            poster: MaybeDynString,
            preload: MaybeDynString,
            src: MaybeDynString,
            width: MaybeDynString, // TODO: int value
        },
        wbr {},
    }

    impl_svg_elements! {
        svg {
            xmlns: MaybeDynString,
        },
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

/// A trait that is implemented for all elements and which provides all the global HTML attributes.
///
/// Reference: <https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes>
#[allow(private_bounds)]
pub trait HtmlGlobalAttributes: IntoHtmlNode + Sized {
    impl_attributes! {
        /// Provides a hint for generating a keyboard shortcut for the current element. This attribute consists of a space-separated list of characters. The browser should use the first one that exists on the computer keyboard layout.
        accesskey: MaybeDynString,
        /// Controls whether inputted text is automatically capitalized and, if so, in what manner.
        autocapitalize: MaybeDynString,
        /// Indicates that an element is to be focused on page load, or as soon as the `<dialog>` it is part of is displayed. This attribute is a boolean, initially false.
        autofocus: MaybeDynBool,
        /// The class global attribute is a space-separated list of the case-sensitive classes of the element.
        /// Classes allow CSS and JavaScript to select and access specific elements via the class selectors.
        class: MaybeDynString,
        /// An enumerated attribute indicating if the element should be editable by the user. If so, the browser modifies its widget to allow editing. The attribute must take one of the following values:
        /// * `true` or the empty string, which indicates that the element must be editable;
        /// * `false`, which indicates that the element must not be editable.
        contenteditable: MaybeDynString,
        /// An enumerated attribute indicating the directionality of the element's text. It can have the following values:
        /// * `ltr`, which means left to right and is to be used for languages that are written from the left to the right (like English);
        /// * `rtl`, which means right to left and is to be used for languages that are written from the right to the left (like Arabic);
        /// * `auto`, which lets the user agent decide. It uses a basic algorithm as it parses the characters inside the element until it finds a character with a strong directionality, then it applies that directionality to the whole element.
        dir: MaybeDynString,
        /// An enumerated attribute indicating whether the element can be dragged, using the Drag and Drop API. It can have the following values:
        /// * `true`, which indicates that the element may be dragged
        /// * `false`, which indicates that the element may not be dragged.
        draggable: MaybeDynString,
        /// Hints what action label (or icon) to present for the enter key on virtual keyboards.
        enterkeyhint: MaybeDynString,
        /// Used to transitively export shadow parts from a nested shadow tree into a containing light tree.
        exportparts: MaybeDynString,
        /// An enumerated attribute indicating that the element is not yet, or is no longer, _relevant_. For example, it can be used to hide elements of the page that can't be used until the login process has been completed. The browser won't render such elements. This attribute must not be used to hide content that could legitimately be shown.
        hidden: MaybeDynBool,
        /// The id global attribute defines an identifier (ID) which must be unique in the whole document. Its purpose is to identify the element when linking (using a fragment identifier), scripting, or styling (with CSS).
        id: MaybeDynString,
        /// A boolean value that makes the browser disregard user input events for the element. Useful when click events are present.
        inert: MaybeDynBool,
        /// Provides a hint to browsers about the type of virtual keyboard configuration to use when editing this element or its contents. Used primarily on `<input>` elements, but is usable on any element while in contenteditable mode.
        inputmode: MaybeDynString,
        /// The is global attribute allows you to specify that a standard HTML element should behave like a defined custom built-in element.
        ///
        /// This attribute can only be used if the specified custom element name has been successfully defined in the current document, and extends the element type it is being applied to.
        is: MaybeDynString,
        /// The unique, global identifier of an item.
        itemid: MaybeDynString,
        /// Used to add properties to an item. Every HTML element may have an `itemprop` attribute specified, where an `itemprop` consists of a name and value pair.
        itemprop: MaybeDynString,
        /// Properties that are not descendants of an element with the `itemscope` attribute can be associated with the item using an `itemref`. It provides a list of element ids (not `itemid`s) with additional properties elsewhere in the document.
        itemref: MaybeDynString,
        /// `itemscope` (usually) works along with `itemtype` to specify that the HTML contained in a block is about a particular item. `itemscope` creates the Item and defines the scope of the `itemtype` associated with it. `itemtype` is a valid URL of a vocabulary (such as schema.org) that describes the item and its properties context.
        itemscope: MaybeDynBool,
        /// Specifies the URL of the vocabulary that will be used to define `itemprops` (item properties) in the data structure. `itemscope` is used to set the scope of where in the data structure the vocabulary set by `itemtype` will be active.
        itemtype: MaybeDynString,
        /// Helps define the language of an element: the language that non-editable elements are in, or the language that editable elements should be written in by the user. The attribute contains one "language tag" (made of hyphen-separated "language subtags") in the format defined in [RFC 5646: Tags for Identifying Languages (also known as BCP 47)](https://datatracker.ietf.org/doc/html/rfc5646). `xml:lang` has priority over it.
        lang: MaybeDynString,
        /// A cryptographic nonce ("number used once") which can be used by Content Security Policy to determine whether or not a given fetch will be allowed to proceed.
        nonce: MaybeDynString,
        /// A space-separated list of the part names of the element. Part names allows CSS to select and style specific elements in a shadow tree via the `::part` pseudo-element.
        part: MaybeDynString,
        /// Used to designate an element as a popover element (see Popover API). Popover elements are hidden via `display: none` until opened via an invoking/control element (i.e. a `<button>` or `<input type="button">` with a popovertarget attribute) or a `HTMLElement.showPopover()` call.
        popover: MaybeDynString,
        /// Roles define the semantic meaning of content, allowing screen readers and other tools to present and support interaction with an object in a way that is consistent with user expectations of that type of object. `roles` are added to HTML elements using `role="role_type"`, where `role_type` is the name of a role in the ARIA specification.
        role: MaybeDynString,
        /// The slot global attribute assigns a slot in a shadow DOM shadow tree to an element: An element with a slot attribute is assigned to the slot created by the `<slot>` element whose name attribute's value matches that slot attribute's value.
        slot: MaybeDynString,
        /// An enumerated attribute defines whether the element may be checked for spelling errors. It may have the following values:
        /// * empty string or `true`, which indicates that the element should be, if possible, checked for spelling errors;
        /// * `false`, which indicates that the element should not be checked for spelling errors.
        spellcheck: MaybeDynString,
        /// Contains CSS styling declarations to be applied to the element. Note that it is recommended for styles to be defined in a separate file or files. This attribute and the `<style>` element have mainly the purpose of allowing for quick styling, for example for testing purposes.
        style: MaybeDynString,
        /// An integer attribute indicating if the element can take input focus (is focusable), if it should participate to sequential keyboard navigation, and if so, at what position. It can take several values:
        /// * a _negative value_ means that the element should be focusable, but should not be reachable via sequential keyboard navigation;
        /// * `0` means that the element should be focusable and reachable via sequential keyboard navigation, but its relative order is defined by the platform convention;
        /// * a _positive value_ means that the element should be focusable and reachable via sequential keyboard navigation; the order in which the elements are focused is the increasing value of the tabindex. If several elements share the same tabindex, their relative order follows their relative positions in the document.
        tabindex: MaybeDynString,
        /// Contains a text representing advisory information related to the element it belongs to. Such information can typically, but not necessarily, be presented to the user as a tooltip.
        title: MaybeDynString,
        /// An enumerated attribute that is used to specify whether an element's attribute values and the values of its Text node children are to be translated when the page is localized, or whether to leave them unchanged. It can have the following values:
        /// * empty string or `yes`, which indicates that the element will be translated.
        /// * `no`, which indicates that the element will not be translated.
        translate: MaybeDynString,
        /// An enumerated attribute used to control the on-screen virtual keyboard behavior on devices such as tablets, mobile phones, or other devices where a hardware keyboard may not be available for elements that its content is editable (for example, it is an `<input>` or `<textarea>` element, or an element with the `contenteditable` attribute set).
        /// `auto` or an _empty string_, which automatically shows the virtual keyboard when the element is focused or tapped.
        /// `manual`, which decouples focus and tap on the element from the virtual keyboard's state.
        virtualkeyboardpolicy: MaybeDynString,
    }
}

/// A trait that is implemented for all SVG elements and which provides all the global SVG
/// attributes.
///
/// Reference: <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute>
#[allow(private_bounds)]
pub trait SvgGlobalAttributes: IntoHtmlNode + Sized {
    impl_attributes! {
        accentHeight("accent-height"): MaybeDynString,
        accumulate: MaybeDynString,
        additive: MaybeDynString,
        alignmentBaseline("alignment-baseline"): MaybeDynString,
        alphabetic: MaybeDynString,
        amplitude: MaybeDynString,
        arabicForm("arabic-form"): MaybeDynString,
        ascent: MaybeDynString,
        attributeName("attributeName"): MaybeDynString,
        attributeType("attributeType"): MaybeDynString,
        azimuth: MaybeDynString,
        baseFrequency("baseFrequency"): MaybeDynString,
        baselineShift("baseline-shift"): MaybeDynString,
        baseProfile("baseProfile"): MaybeDynString,
        bbox: MaybeDynString,
        begin: MaybeDynString,
        bias: MaybeDynString,
        by: MaybeDynString,
        calcMode("calcMode"): MaybeDynString,
        capHeight("cap-height"): MaybeDynString,
        class: MaybeDynString,
        clipPathUnits("clipPathUnits"): MaybeDynString,
        clipPath("clip-path"): MaybeDynString,
        clipRule("clip-rule"): MaybeDynString,
        color: MaybeDynString,
        colorInterpolation("color-interpolation"): MaybeDynString,
        colorInterpolationFilters("color-interpolation-filters"): MaybeDynString,
        colorProfile("color-profile"): MaybeDynString,
        colorRendering("color-rendering"): MaybeDynString,
        crossorigin: MaybeDynString,
        cursor: MaybeDynString,
        cx: MaybeDynString,
        cy: MaybeDynString,
        d: MaybeDynString,
        decelerate: MaybeDynString,
        descent: MaybeDynString,
        diffuseConstant("diffuseConstant"): MaybeDynString,
        direction: MaybeDynString,
        display: MaybeDynString,
        divisor: MaybeDynString,
        dominantBaseline("dominant-baseline"): MaybeDynString,
        dur: MaybeDynString,
        dx: MaybeDynString,
        dy: MaybeDynString,
        edgeMode("edgeMode"): MaybeDynString,
        elevation: MaybeDynString,
        enableBackground("enable-background"): MaybeDynString,
        end: MaybeDynString,
        exponent: MaybeDynString,
        fill: MaybeDynString,
        fillOpacity("fill-opacity"): MaybeDynString,
        fillRule("fill-rule"): MaybeDynString,
        filter: MaybeDynString,
        filterUnits("filterUnits"): MaybeDynString,
        floodColor("flood-color"): MaybeDynString,
        floodOpacity("flood-opacity"): MaybeDynString,
        fontFamily("font-family"): MaybeDynString,
        fontSize("font-size"): MaybeDynString,
        fontSizeAdjust("font-size-adjust"): MaybeDynString,
        fontStretch("font-stretch"): MaybeDynString,
        fontStyle("font-style"): MaybeDynString,
        fontVariant("font-variant"): MaybeDynString,
        fontWeight("font-weight"): MaybeDynString,
        format: MaybeDynString,
        from: MaybeDynString,
        fr: MaybeDynString,
        fx: MaybeDynString,
        fy: MaybeDynString,
        g1: MaybeDynString,
        g2: MaybeDynString,
        glyphName("glyph-name"): MaybeDynString,
        glyphOrientationHorizontal("glyph-orientation-horizontal"): MaybeDynString,
        glyphOrientationVertical("glyph-orientation-vertical"): MaybeDynString,
        glyphRef: MaybeDynString,
        gradientTransform("gradientTransform"): MaybeDynString,
        gradientUnits("gradientUnits"): MaybeDynString,
        hanging: MaybeDynString,
        height: MaybeDynString,
        href: MaybeDynString,
        hreflang: MaybeDynString,
        horizAdvX("horiz-adv-x"): MaybeDynString,
        horizOriginX("horiz-origin-x"): MaybeDynString,
        id: MaybeDynString,
        ideographic: MaybeDynString,
        imageRendering("image-rendering"): MaybeDynString,
        in_: MaybeDynString,
        in2: MaybeDynString,
        intercept: MaybeDynString,
        k: MaybeDynString,
        k1: MaybeDynString,
        k2: MaybeDynString,
        k3: MaybeDynString,
        k4: MaybeDynString,
        kernelMatrix("kernelMatrix"): MaybeDynString,
        kernelUnitLength("kernelUnitLength"): MaybeDynString,
        kerning: MaybeDynString,
        keyPoints("keyPoints"): MaybeDynString,
        keySplines("keySplines"): MaybeDynString,
        keyTimes("keyTimes"): MaybeDynString,
        lang: MaybeDynString,
        lengthAdjust("lengthAdjust"): MaybeDynString,
        letterSpacing("letter-spacing"): MaybeDynString,
        lightingColor("lighting-color"): MaybeDynString,
        limitingConeAngle("limitingConeAngle"): MaybeDynString,
        local: MaybeDynString,
        markerEnd("marker-end"): MaybeDynString,
        markerMid("marker-mid"): MaybeDynString,
        markerStart("marker-start"): MaybeDynString,
        markerHeight("markerHeight"): MaybeDynString,
        markerUnits("markerUnits"): MaybeDynString,
        markerWidth("markerWidth"): MaybeDynString,
        mask: MaybeDynString,
        maskContentUnits("maskContentUnits"): MaybeDynString,
        maskUnits("maskUnits"): MaybeDynString,
        mathematical: MaybeDynString,
        max: MaybeDynString,
        media: MaybeDynString,
        method: MaybeDynString,
        min: MaybeDynString,
        mode: MaybeDynString,
        name: MaybeDynString,
        numOctaves("numOctaves"): MaybeDynString,
        offset: MaybeDynString,
        opacity: MaybeDynString,
        operator: MaybeDynString,
        order: MaybeDynString,
        orient: MaybeDynString,
        orientation: MaybeDynString,
        origin: MaybeDynString,
        overflow: MaybeDynString,
        overlinePosition("overline-position"): MaybeDynString,
        overlineThickness("overline-thickness"): MaybeDynString,
        panose1("panose-1"): MaybeDynString,
        paintOrder("paint-order"): MaybeDynString,
        path: MaybeDynString,
        pathLength("pathLength"): MaybeDynString,
        patternContentUnits("patternContentUnits"): MaybeDynString,
        patternTransform("patternTransform"): MaybeDynString,
        patternUnits("patternUnits"): MaybeDynString,
        ping: MaybeDynString,
        pointerEvents("pointer-events"): MaybeDynString,
        points: MaybeDynString,
        pointsAtX("pointsAtX"): MaybeDynString,
        pointsAtY("pointsAtY"): MaybeDynString,
        pointsAtZ("pointsAtZ"): MaybeDynString,
        preserveAlpha("preserveAlpha"): MaybeDynString,
        preserveAspectRatio("preserveAspectRatio"): MaybeDynString,
        primitiveUnits("primitiveUnits"): MaybeDynString,
        r: MaybeDynString,
        radius: MaybeDynString,
        referrerPolicy("referrerPolicy"): MaybeDynString,
        refX("refX"): MaybeDynString,
        refY("refY"): MaybeDynString,
        rel: MaybeDynString,
        renderingIntent("rendering-intent"): MaybeDynString,
        repeatCount("repeatCount"): MaybeDynString,
        repeatDur("repeatDur"): MaybeDynString,
        requiredExtensions("requiredExtensions"): MaybeDynString,
        requiredFeatures("requiredFeatures"): MaybeDynString,
        restart: MaybeDynString,
        result: MaybeDynString,
        rotate: MaybeDynString,
        rx: MaybeDynString,
        ry: MaybeDynString,
        scale: MaybeDynString,
        seed: MaybeDynString,
        shapeRendering("shape-rendering"): MaybeDynString,
        slope: MaybeDynString,
        spacing: MaybeDynString,
        specularConstant("specularConstant"): MaybeDynString,
        specularExponent("specularExponent"): MaybeDynString,
        speed: MaybeDynString,
        spreadMethod("spreadMethod"): MaybeDynString,
        startOffset("startOffset"): MaybeDynString,
        stdDeviation("stdDeviation"): MaybeDynString,
        stemh: MaybeDynString,
        stemv: MaybeDynString,
        stitchTiles("stitchTiles"): MaybeDynString,
        stopColor("stop-color"): MaybeDynString,
        stopOpacity("stop-opacity"): MaybeDynString,
        strikethroughPosition("strikethrough-position"): MaybeDynString,
        strikethroughThickness("strikethrough-thickness"): MaybeDynString,
        string: MaybeDynString,
        stroke: MaybeDynString,
        strokeDasharray("stroke-dasharray"): MaybeDynString,
        strokeDashoffset("stroke-dashoffset"): MaybeDynString,
        strokeLinecap("stroke-linecap"): MaybeDynString,
        strokeLinejoin("stroke-linejoin"): MaybeDynString,
        strokeMiterlimit("stroke-miterlimit"): MaybeDynString,
        strokeOpacity("stroke-opacity"): MaybeDynString,
        strokeWidth("stroke-width"): MaybeDynString,
        style: MaybeDynString,
        surfaceScale("surfaceScale"): MaybeDynString,
        systemLanguage("systemLanguage"): MaybeDynString,
        tabindex: MaybeDynString,
        tableValues("tableValues"): MaybeDynString,
        target: MaybeDynString,
        targetX("targetX"): MaybeDynString,
        targetY("targetY"): MaybeDynString,
        textAnchor("text-anchor"): MaybeDynString,
        textDecoration("text-decoration"): MaybeDynString,
        textRendering("text-rendering"): MaybeDynString,
        textLength("textLength"): MaybeDynString,
        to: MaybeDynString,
        transform: MaybeDynString,
        transformOrigin("transform-origin"): MaybeDynString,
        type_: MaybeDynString,
        u1: MaybeDynString,
        u2: MaybeDynString,
        underlinePosition("underline-position"): MaybeDynString,
        underlineThickness("underline-thickness"): MaybeDynString,
        unicode: MaybeDynString,
        unicodeBidi("unicode-bidi"): MaybeDynString,
        unicodeRange("unicode-range"): MaybeDynString,
        unitsPerEm("units-per-em"): MaybeDynString,
        vAlphabetic("v-alphabetic"): MaybeDynString,
        vHanging("v-hanging"): MaybeDynString,
        vIdeographic("v-ideographic"): MaybeDynString,
        vMathematical("v-mathematical"): MaybeDynString,
        values: MaybeDynString,
        vectorEffect("vector-effect"): MaybeDynString,
        version: MaybeDynString,
        vertAdvY("vert-adv-y"): MaybeDynString,
        vertOriginX("vert-origin-x"): MaybeDynString,
        vertOriginY("vert-origin-y"): MaybeDynString,
        viewBox: MaybeDynString,
        visibility: MaybeDynString,
        width: MaybeDynString,
        widths: MaybeDynString,
        wordSpacing("word-spacing"): MaybeDynString,
        writingMode("writing-mode"): MaybeDynString,
        x: MaybeDynString,
        xHeight("x-height"): MaybeDynString,
        x1: MaybeDynString,
        x2: MaybeDynString,
        xChannelSelector("xChannelSelector"): MaybeDynString,
        xmlBase("xml:base"): MaybeDynString,
        xmlLang("xml:lang"): MaybeDynString,
        xmlSpace("xml:space"): MaybeDynString,
        y: MaybeDynString,
        y1: MaybeDynString,
        y2: MaybeDynString,
        yChannelSelector("yChannelSelector"): MaybeDynString,
        zoomAndPan("zoomAndPan"): MaybeDynString,
    }
}

/// Attributes that are available on all elements.
#[allow(private_bounds)]
pub trait GlobalAttributes: IntoHtmlNode + Sized {
    /// Set attribute `name` with `value`.
    fn attr(mut self, name: &'static str, value: impl Into<MaybeDynString>) -> Self {
        let node = self.as_html_node_mut();
        set_attribute(node, name, value.into());
        self
    }

    /// Set attribute `name` with `value`.
    fn bool_attr(mut self, name: &'static str, value: impl Into<MaybeDynBool>) -> Self {
        let node = self.as_html_node_mut();
        set_attribute(node, name, value.into());
        self
    }

    /// Set JS property `name` with `value`.
    fn prop(mut self, name: &'static str, value: impl Into<MaybeDynJsValue>) -> Self {
        let node = self.as_html_node_mut();
        set_attribute(node, name, value.into());
        self
    }

    /// Set an event handler with `name`.
    fn on<T: events::EventDescriptor, R>(
        mut self,
        _: T,
        mut handler: impl EventHandler<T, R>,
    ) -> Self {
        let scope = use_current_scope(); // Run handler inside the current scope.
        let handler = move |ev: web_sys::Event| scope.run_in(|| handler.call(ev.unchecked_into()));
        let node = self.as_html_node_mut();
        node.set_event_handler(T::NAME.into(), handler);
        self
    }

    fn bind<T: bind::BindDescriptor>(mut self, _: T, signal: Signal<T::ValueTy>) -> Self {
        let node = self.as_html_node_mut();
        let scope = use_current_scope(); // Run handler inside the current scope.
        let handler = move |ev: web_sys::Event| {
            scope.run_in(|| {
                let value =
                    js_sys::Reflect::get(&ev.current_target().unwrap(), &T::TARGET_PROPERTY.into())
                        .unwrap();
                signal.set(T::CONVERT_FROM_JS(&value).expect("failed to convert value from js"));
            })
        };
        node.set_event_handler(<T::Event as events::EventDescriptor>::NAME.into(), handler);

        self.prop(T::TARGET_PROPERTY, move || signal.get_clone().into())
    }

    /// Set the inner html of an element.
    fn dangerously_set_inner_html(mut self, inner_html: impl Into<Cow<'static, str>>) -> Self {
        self.as_html_node_mut().set_inner_html(inner_html.into());
        self
    }

    /// Set the children of an element.
    fn children(mut self, children: impl Into<View>) -> Self {
        self.as_html_node_mut().append_view(children.into());
        self
    }

    /// Set a [`NodeRef`] on this element.
    fn r#ref(self, noderef: NodeRef) -> Self {
        if is_not_ssr!() {
            noderef.set(Some(self.as_html_node().as_web_sys().clone()));
        }
        self
    }
}

/// Helper function for setting a dynamic attribute.
fn set_attribute(el: &mut HtmlNode, name: &'static str, value: impl AttributeValue) {
    value.set_self(el, name);
}

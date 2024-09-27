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

impl AsHtmlNode for CustomElement {
    fn as_html_node(&mut self) -> &mut HtmlNode {
        &mut self.0
    }
}

impl GlobalProps for CustomElement {}
impl HtmlGlobalAttributes for CustomElement {}

macro_rules! impl_attribute {
    ($(#[$attr:meta])* $v:vis $ident:ident: $ty:ty) => {
        impl_attribute!($(#[$attr])* $v $ident (stringify!($ident)): $ty);
    };
    ($(#[$attr:meta])* $v:vis $ident:ident ($name:expr): $ty:ty) => {
        $(#[$attr])*
        $v fn $ident(mut self, value: $ty) -> Self {
            self.set_attribute($name, value.into_maybe_dyn());
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

            impl AsHtmlNode for [<Html $name:camel>] {
                fn as_html_node(&mut self) -> &mut HtmlNode {
                    &mut self.0
                }
            }

            impl GlobalProps for [<Html $name:camel>] {}
            impl HtmlGlobalAttributes for [<Html $name:camel>] {}

            #[doc = "Trait that provides attributes for the `<" $name ">` HTML element."]
            pub trait [<Html $name:camel Attributes>]: SetAttribute + Sized {
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

            impl AsHtmlNode for [<Svg $name:camel>] {
                fn as_html_node(&mut self) -> &mut HtmlNode {
                    &mut self.0
                }
            }

            impl GlobalProps for [<Svg $name:camel>] {}
            impl SvgGlobalAttributes for [<Svg $name:camel>] {}

            #[doc = "Trait that provides attributes for the `<" $name ">` SVG element."]
            pub trait [<Svg $name:camel Attributes>]: SetAttribute + Sized {
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
            download: impl IntoMaybeDynCowStr,
            href: impl IntoMaybeDynCowStr,
            hreflang: impl IntoMaybeDynCowStr,
            target: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
            ping: impl IntoMaybeDynCowStr,
            rel: impl IntoMaybeDynCowStr,
        },
        abbr {},
        address {},
        area {
            alt: impl IntoMaybeDynCowStr,
            coords: impl IntoMaybeDynCowStr,
            download: impl IntoMaybeDynCowStr,
            href: impl IntoMaybeDynCowStr,
            hreflang: impl IntoMaybeDynCowStr,
            media: impl IntoMaybeDynCowStr,
            referrerpolicy: impl IntoMaybeDynCowStr,
            ping: impl IntoMaybeDynCowStr,
            rel: impl IntoMaybeDynCowStr,
            shape: impl IntoMaybeDynCowStr,
            target: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
        },
        article {},
        aside {},
        audio {
            autoplay: impl IntoMaybeDynBool,
            controls: impl IntoMaybeDynBool,
            crossorigin: impl IntoMaybeDynCowStr,
            muted: impl IntoMaybeDynBool,
            preload: impl IntoMaybeDynCowStr,
            src: impl IntoMaybeDynCowStr,
            r#loop("loop"): impl IntoMaybeDynBool,
        },
        b {},
        base {
            href: impl IntoMaybeDynCowStr,
            target: impl IntoMaybeDynCowStr,
        },
        bdi {},
        bdo {},
        blockquote {
            cite: impl IntoMaybeDynCowStr,
        },
        body {},
        br {},
        /// The `<button>` HTML element represents a clickable button, used to submit forms or anywhere in a document for accessible, standard button functionality.
        ///
        /// By default, HTML buttons are presented in a style resembling the platform the user agent runs on, but you can change buttons’ appearance with CSS.
        button {
            autofocus: impl IntoMaybeDynBool,
            disabled: impl IntoMaybeDynBool,
            form: impl IntoMaybeDynCowStr,
            formaction: impl IntoMaybeDynCowStr,
            formenctype: impl IntoMaybeDynCowStr,
            formmethod: impl IntoMaybeDynCowStr,
            formnovalidate: impl IntoMaybeDynBool,
            formtarget: impl IntoMaybeDynCowStr,
            name: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
            value: impl IntoMaybeDynCowStr,
        },
        canvas {
            height: impl IntoMaybeDynCowStr, // TODO: int value
            width: impl IntoMaybeDynCowStr, // TODO: int value
        },
        caption {},
        cite {},
        code {
            language: impl IntoMaybeDynCowStr,
        },
        col {
            span: impl IntoMaybeDynCowStr, // TODO: int value
        },
        colgroup {
            span: impl IntoMaybeDynCowStr, // TODO: int value
        },
        data {
            value: impl IntoMaybeDynCowStr,
        },
        datalist {},
        dd {},
        del {
            cite: impl IntoMaybeDynCowStr,
            datetime: impl IntoMaybeDynCowStr,
        },
        details {
            open: impl IntoMaybeDynBool,
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
            height: impl IntoMaybeDynCowStr,
            src: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
            width: impl IntoMaybeDynCowStr, // TODO: int value
        },
        fieldset {},
        figcaption {},
        figure {},
        footer {},
        form {
            acceptcharset: impl IntoMaybeDynCowStr,
            action: impl IntoMaybeDynCowStr,
            autocomplete: impl IntoMaybeDynCowStr,
            enctype: impl IntoMaybeDynCowStr,
            method: impl IntoMaybeDynCowStr,
            name: impl IntoMaybeDynCowStr,
            novalidate: impl IntoMaybeDynBool,
            target: impl IntoMaybeDynCowStr,
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
            allow: impl IntoMaybeDynCowStr,
            allowfullscreen: impl IntoMaybeDynBool,
            allowpaymentrequest: impl IntoMaybeDynBool,
            height: impl IntoMaybeDynCowStr,
            loading: impl IntoMaybeDynCowStr,
            name: impl IntoMaybeDynCowStr,
            referrerpolicy: impl IntoMaybeDynCowStr,
            sandbox: impl IntoMaybeDynBool,
            src: impl IntoMaybeDynCowStr,
            srcdoc: impl IntoMaybeDynCowStr,
            width: impl IntoMaybeDynCowStr,
        },
        img {
            alt: impl IntoMaybeDynCowStr,
            crossorigin: impl IntoMaybeDynCowStr,
            decoding: impl IntoMaybeDynCowStr,
            height: impl IntoMaybeDynCowStr,
            ismap: impl IntoMaybeDynBool,
            loading: impl IntoMaybeDynCowStr,
            referrerpolicy: impl IntoMaybeDynCowStr,
            sizes: impl IntoMaybeDynCowStr,
            src: impl IntoMaybeDynCowStr,
            srcset: impl IntoMaybeDynCowStr,
            usemap: impl IntoMaybeDynCowStr,
            width: impl IntoMaybeDynCowStr,
        },
        /// The `<input>` HTML element is used to create interactive controls for web-based forms in order to accept data from the user; a wide variety of types of input data and control widgets are available, depending on the device and user agent. The `<input>` element is one of the most powerful and complex in all of HTML due to the sheer number of combinations of input types and attributes.
        input {
            accept: impl IntoMaybeDynCowStr,
            alt: impl IntoMaybeDynCowStr,
            autocomplete: impl IntoMaybeDynCowStr,
            autofocus: impl IntoMaybeDynBool,
            capture: impl IntoMaybeDynCowStr,
            checked: impl IntoMaybeDynBool,
            directory: impl IntoMaybeDynCowStr,
            disabled: impl IntoMaybeDynBool,
            form: impl IntoMaybeDynCowStr,
            formaction: impl IntoMaybeDynCowStr,
            formenctype: impl IntoMaybeDynCowStr,
            formmethod: impl IntoMaybeDynCowStr,
            formnovalidate: impl IntoMaybeDynBool,
            formtarget: impl IntoMaybeDynCowStr,
            height: impl IntoMaybeDynCowStr, // TODO: int value
            initial_checked: impl IntoMaybeDynBool,
            initial_value: impl IntoMaybeDynCowStr,
            list: impl IntoMaybeDynCowStr,
            max: impl IntoMaybeDynCowStr,
            maxlength: impl IntoMaybeDynCowStr, // TODO: int value
            min: impl IntoMaybeDynCowStr,
            minlength: impl IntoMaybeDynCowStr, // TODO: int value
            multiple: impl IntoMaybeDynBool,
            name: impl IntoMaybeDynCowStr,
            pattern: impl IntoMaybeDynCowStr,
            placeholder: impl IntoMaybeDynCowStr,
            readonly: impl IntoMaybeDynBool,
            required: impl IntoMaybeDynBool,
            size: impl IntoMaybeDynCowStr, // TODO: int value
            spellcheck: impl IntoMaybeDynBool,
            src: impl IntoMaybeDynCowStr,
            step: impl IntoMaybeDynCowStr,
            tabindex: impl IntoMaybeDynCowStr, // TODO: int value
            r#type("type"): impl IntoMaybeDynCowStr,
            value: impl IntoMaybeDynCowStr,
            width: impl IntoMaybeDynCowStr, // TODO: int value
        },
        ins {
            cite: impl IntoMaybeDynCowStr,
            datetime: impl IntoMaybeDynCowStr,
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
            form: impl IntoMaybeDynCowStr,
            r#for("for"): impl IntoMaybeDynCowStr,
        },
        legend {},
        /// The `<li>` HTML element is used to represent an item in a list. It must be contained in a parent element: an ordered list (`<ol>`), an unordered list (`<ul>`), or a menu (`<menu>`). In menus and unordered lists, list items are usually displayed using bullet points. In ordered lists, they are usually displayed with an ascending counter on the left, such as a number or letter.
        li {
            value: impl IntoMaybeDynCowStr, // TODO: int value
        },
        link {
            r#as("as"): impl IntoMaybeDynCowStr,
            crossorigin: impl IntoMaybeDynCowStr,
            href: impl IntoMaybeDynCowStr,
            hreflang: impl IntoMaybeDynCowStr,
            media: impl IntoMaybeDynCowStr,
            rel: impl IntoMaybeDynCowStr,
            sizes: impl IntoMaybeDynCowStr,
            title: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
            integrity: impl IntoMaybeDynCowStr,
        },
        main {},
        map {
            name: impl IntoMaybeDynCowStr,
        },
        mark {},
        menu {},
        menuitem {},
        meta {
            charset: impl IntoMaybeDynCowStr,
            content: impl IntoMaybeDynCowStr,
            http_equiv("http-equiv"): impl IntoMaybeDynCowStr,
            name: impl IntoMaybeDynCowStr,
        },
        meter {
            value: impl IntoMaybeDynCowStr, // TODO: int value
            min: impl IntoMaybeDynCowStr, // TODO: int value
            max: impl IntoMaybeDynCowStr, // TODO: int value
            low: impl IntoMaybeDynCowStr, // TODO: int value
            high: impl IntoMaybeDynCowStr, // TODO: int value
            optimum: impl IntoMaybeDynCowStr, // TODO: int value
            form: impl IntoMaybeDynCowStr,
        },
        nav {},
        noscript {},
        object {
            data: impl IntoMaybeDynCowStr,
            form: impl IntoMaybeDynCowStr,
            height: impl IntoMaybeDynCowStr, // TODO: int value
            name: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
            typemustmatch: impl IntoMaybeDynBool,
            usemap: impl IntoMaybeDynCowStr,
            width: impl IntoMaybeDynCowStr,
        },
        /// The `<ol>` HTML element represents an ordered list of items — typically rendered as a numbered list.
        ol {
            reversed: impl IntoMaybeDynBool,
            start: impl IntoMaybeDynCowStr, // TODO: int value
            r#type("type"): impl IntoMaybeDynCowStr,
        },
        optgroup {
            disabled: impl IntoMaybeDynBool,
            label: impl IntoMaybeDynCowStr,
        },
        option {
            disabled: impl IntoMaybeDynBool,
            initial_selected: impl IntoMaybeDynBool,
            label: impl IntoMaybeDynCowStr,
            selected: impl IntoMaybeDynBool,
            value: impl IntoMaybeDynCowStr,
        },
        output {
            r#for("for"): impl IntoMaybeDynCowStr,
            form: impl IntoMaybeDynCowStr,
            name: impl IntoMaybeDynCowStr,
        },
        /// The `<p>` HTML element represents a paragraph. Paragraphs are usually represented in visual media as blocks of text separated from adjacent blocks by blank lines and/or first-line indentation, but HTML paragraphs can be any structural grouping of related content, such as images or form fields.
        ///
        /// Paragraphs are block-level elements, and notably will automatically close if another block-level element is parsed before the closing `</p>` tag.
        p {},
        param {
            name: impl IntoMaybeDynCowStr,
            value: impl IntoMaybeDynCowStr,
        },
        picture {},
        pre {},
        progress {
            value: impl IntoMaybeDynCowStr, // TODO: f64 value
            max: impl IntoMaybeDynCowStr, // TODO: f64 value
        },
        q {
            cite: impl IntoMaybeDynCowStr,
        },
        rp {},
        rt {},
        ruby {},
        s {},
        samp {},
        script {
            r#async: impl IntoMaybeDynBool,
            crossorigin: impl IntoMaybeDynCowStr,
            defer: impl IntoMaybeDynBool,
            integrity: impl IntoMaybeDynCowStr,
            nomodule: impl IntoMaybeDynBool,
            nonce: impl IntoMaybeDynCowStr,
            src: impl IntoMaybeDynCowStr,
            script: impl IntoMaybeDynCowStr,
            text: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
        },
        section {},
        select {
            autocomplete: impl IntoMaybeDynCowStr,
            autofocus: impl IntoMaybeDynBool,
            disabled: impl IntoMaybeDynBool,
            form: impl IntoMaybeDynCowStr,
            multiple: impl IntoMaybeDynBool,
            name: impl IntoMaybeDynCowStr,
            required: impl IntoMaybeDynBool,
            size: impl IntoMaybeDynCowStr, // TODO: int value
            value: impl IntoMaybeDynCowStr,
        },
        small {},
        source {
            src: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
        },
        /// The `<span>` HTML element is a generic inline container for phrasing content, which does not inherently represent anything. It can be used to group elements for styling purposes (using the class or id attributes), or because they share attribute values, such as lang. It should be used only when no other semantic element is appropriate. `<span>` is very much like a `<div>` element, but `<div>` is a block-level element whereas a `<span>` is an inline element.
        span {},
        strong {},
        style {
            media: impl IntoMaybeDynCowStr,
            nonce: impl IntoMaybeDynCowStr,
            title: impl IntoMaybeDynCowStr,
            r#type("type"): impl IntoMaybeDynCowStr,
        },
        sub {},
        summary {},
        sup {},
        table {},
        tbody {},
        td {
            colspan: impl IntoMaybeDynCowStr, // TODO: int value
            headers: impl IntoMaybeDynCowStr,
            rowspan: impl IntoMaybeDynCowStr, // TODO: int value
        },
        template {},
        textarea {
            autocomplete: impl IntoMaybeDynCowStr,
            autofocus: impl IntoMaybeDynBool,
            cols: impl IntoMaybeDynCowStr, // TODO: int value
            disabled: impl IntoMaybeDynBool,
            form: impl IntoMaybeDynCowStr,
            initial_value: impl IntoMaybeDynCowStr,
            maxlength: impl IntoMaybeDynCowStr, // TODO: int value
            minlength: impl IntoMaybeDynCowStr, // TODO: int value
            name: impl IntoMaybeDynCowStr,
            placeholder: impl IntoMaybeDynCowStr,
            readonly: impl IntoMaybeDynBool,
            required: impl IntoMaybeDynBool,
            rows: impl IntoMaybeDynCowStr, // TODO: int value
            spellcheck: impl IntoMaybeDynBool,
            r#type("type"): impl IntoMaybeDynCowStr,
            value: impl IntoMaybeDynCowStr,
            wrap: impl IntoMaybeDynCowStr,
        },
        tfoot {},
        th {
            abbr: impl IntoMaybeDynCowStr,
            colspan: impl IntoMaybeDynCowStr, // TODO: int value
            headers: impl IntoMaybeDynCowStr,
            rowspan: impl IntoMaybeDynCowStr, // TODO: int value
            scope: impl IntoMaybeDynCowStr,
        },
        thead {},
        time {
            datetime: impl IntoMaybeDynCowStr,
        },
        title {},
        tr {},
        track {
            default: impl IntoMaybeDynBool,
            kind: impl IntoMaybeDynCowStr,
            label: impl IntoMaybeDynCowStr,
            src: impl IntoMaybeDynCowStr,
            srclang: impl IntoMaybeDynCowStr,
        },
        u {},
        /// The `<ul>` HTML element represents an unordered list of items, typically rendered as a bulleted list.
        ul {},
        var {},
        video {
            autoplay: impl IntoMaybeDynBool,
            controls: impl IntoMaybeDynBool,
            crossorigin: impl IntoMaybeDynCowStr,
            height: impl IntoMaybeDynCowStr, // TODO: int value
            r#loop("loop"): impl IntoMaybeDynBool,
            muted: impl IntoMaybeDynBool,
            playsinline: impl IntoMaybeDynBool,
            poster: impl IntoMaybeDynCowStr,
            preload: impl IntoMaybeDynCowStr,
            src: impl IntoMaybeDynCowStr,
            width: impl IntoMaybeDynCowStr, // TODO: int value
        },
        wbr {},
    }

    impl_svg_elements! {
        svg {
            xmlns: impl IntoMaybeDynCowStr,
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
pub trait HtmlGlobalAttributes: SetAttribute + Sized {
    impl_attributes! {
        /// Provides a hint for generating a keyboard shortcut for the current element. This attribute consists of a space-separated list of characters. The browser should use the first one that exists on the computer keyboard layout.
        accesskey: impl IntoMaybeDynCowStr,
        /// Controls whether inputted text is automatically capitalized and, if so, in what manner.
        autocapitalize: impl IntoMaybeDynCowStr,
        /// Indicates that an element is to be focused on page load, or as soon as the `<dialog>` it is part of is displayed. This attribute is a boolean, initially false.
        autofocus: impl IntoMaybeDynBool,
        /// The class global attribute is a space-separated list of the case-sensitive classes of the element.
        /// Classes allow CSS and JavaScript to select and access specific elements via the class selectors.
        class: impl IntoMaybeDynCowStr,
        /// An enumerated attribute indicating if the element should be editable by the user. If so, the browser modifies its widget to allow editing. The attribute must take one of the following values:
        /// * `true` or the empty string, which indicates that the element must be editable;
        /// * `false`, which indicates that the element must not be editable.
        contenteditable: impl IntoMaybeDynCowStr,
        /// An enumerated attribute indicating the directionality of the element's text. It can have the following values:
        /// * `ltr`, which means left to right and is to be used for languages that are written from the left to the right (like English);
        /// * `rtl`, which means right to left and is to be used for languages that are written from the right to the left (like Arabic);
        /// * `auto`, which lets the user agent decide. It uses a basic algorithm as it parses the characters inside the element until it finds a character with a strong directionality, then it applies that directionality to the whole element.
        dir: impl IntoMaybeDynCowStr,
        /// An enumerated attribute indicating whether the element can be dragged, using the Drag and Drop API. It can have the following values:
        /// * `true`, which indicates that the element may be dragged
        /// * `false`, which indicates that the element may not be dragged.
        draggable: impl IntoMaybeDynCowStr,
        /// Hints what action label (or icon) to present for the enter key on virtual keyboards.
        enterkeyhint: impl IntoMaybeDynCowStr,
        /// Used to transitively export shadow parts from a nested shadow tree into a containing light tree.
        exportparts: impl IntoMaybeDynCowStr,
        /// An enumerated attribute indicating that the element is not yet, or is no longer, _relevant_. For example, it can be used to hide elements of the page that can't be used until the login process has been completed. The browser won't render such elements. This attribute must not be used to hide content that could legitimately be shown.
        hidden: impl IntoMaybeDynBool,
        /// The id global attribute defines an identifier (ID) which must be unique in the whole document. Its purpose is to identify the element when linking (using a fragment identifier), scripting, or styling (with CSS).
        id: impl IntoMaybeDynCowStr,
        /// A boolean value that makes the browser disregard user input events for the element. Useful when click events are present.
        inert: impl IntoMaybeDynBool,
        /// Provides a hint to browsers about the type of virtual keyboard configuration to use when editing this element or its contents. Used primarily on `<input>` elements, but is usable on any element while in contenteditable mode.
        inputmode: impl IntoMaybeDynCowStr,
        /// The is global attribute allows you to specify that a standard HTML element should behave like a defined custom built-in element.
        ///
        /// This attribute can only be used if the specified custom element name has been successfully defined in the current document, and extends the element type it is being applied to.
        is: impl IntoMaybeDynCowStr,
        /// The unique, global identifier of an item.
        itemid: impl IntoMaybeDynCowStr,
        /// Used to add properties to an item. Every HTML element may have an `itemprop` attribute specified, where an `itemprop` consists of a name and value pair.
        itemprop: impl IntoMaybeDynCowStr,
        /// Properties that are not descendants of an element with the `itemscope` attribute can be associated with the item using an `itemref`. It provides a list of element ids (not `itemid`s) with additional properties elsewhere in the document.
        itemref: impl IntoMaybeDynCowStr,
        /// `itemscope` (usually) works along with `itemtype` to specify that the HTML contained in a block is about a particular item. `itemscope` creates the Item and defines the scope of the `itemtype` associated with it. `itemtype` is a valid URL of a vocabulary (such as schema.org) that describes the item and its properties context.
        itemscope: impl IntoMaybeDynBool,
        /// Specifies the URL of the vocabulary that will be used to define `itemprops` (item properties) in the data structure. `itemscope` is used to set the scope of where in the data structure the vocabulary set by `itemtype` will be active.
        itemtype: impl IntoMaybeDynCowStr,
        /// Helps define the language of an element: the language that non-editable elements are in, or the language that editable elements should be written in by the user. The attribute contains one "language tag" (made of hyphen-separated "language subtags") in the format defined in [RFC 5646: Tags for Identifying Languages (also known as BCP 47)](https://datatracker.ietf.org/doc/html/rfc5646). `xml:lang` has priority over it.
        lang: impl IntoMaybeDynCowStr,
        /// A cryptographic nonce ("number used once") which can be used by Content Security Policy to determine whether or not a given fetch will be allowed to proceed.
        nonce: impl IntoMaybeDynCowStr,
        /// A space-separated list of the part names of the element. Part names allows CSS to select and style specific elements in a shadow tree via the `::part` pseudo-element.
        part: impl IntoMaybeDynCowStr,
        /// Used to designate an element as a popover element (see Popover API). Popover elements are hidden via `display: none` until opened via an invoking/control element (i.e. a `<button>` or `<input type="button">` with a popovertarget attribute) or a `HTMLElement.showPopover()` call.
        popover: impl IntoMaybeDynCowStr,
        /// Roles define the semantic meaning of content, allowing screen readers and other tools to present and support interaction with an object in a way that is consistent with user expectations of that type of object. `roles` are added to HTML elements using `role="role_type"`, where `role_type` is the name of a role in the ARIA specification.
        role: impl IntoMaybeDynCowStr,
        /// The slot global attribute assigns a slot in a shadow DOM shadow tree to an element: An element with a slot attribute is assigned to the slot created by the `<slot>` element whose name attribute's value matches that slot attribute's value.
        slot: impl IntoMaybeDynCowStr,
        /// An enumerated attribute defines whether the element may be checked for spelling errors. It may have the following values:
        /// * empty string or `true`, which indicates that the element should be, if possible, checked for spelling errors;
        /// * `false`, which indicates that the element should not be checked for spelling errors.
        spellcheck: impl IntoMaybeDynCowStr,
        /// Contains CSS styling declarations to be applied to the element. Note that it is recommended for styles to be defined in a separate file or files. This attribute and the `<style>` element have mainly the purpose of allowing for quick styling, for example for testing purposes.
        style: impl IntoMaybeDynCowStr,
        /// An integer attribute indicating if the element can take input focus (is focusable), if it should participate to sequential keyboard navigation, and if so, at what position. It can take several values:
        /// * a _negative value_ means that the element should be focusable, but should not be reachable via sequential keyboard navigation;
        /// * `0` means that the element should be focusable and reachable via sequential keyboard navigation, but its relative order is defined by the platform convention;
        /// * a _positive value_ means that the element should be focusable and reachable via sequential keyboard navigation; the order in which the elements are focused is the increasing value of the tabindex. If several elements share the same tabindex, their relative order follows their relative positions in the document.
        tabindex: impl IntoMaybeDynCowStr,
        /// Contains a text representing advisory information related to the element it belongs to. Such information can typically, but not necessarily, be presented to the user as a tooltip.
        title: impl IntoMaybeDynCowStr,
        /// An enumerated attribute that is used to specify whether an element's attribute values and the values of its Text node children are to be translated when the page is localized, or whether to leave them unchanged. It can have the following values:
        /// * empty string or `yes`, which indicates that the element will be translated.
        /// * `no`, which indicates that the element will not be translated.
        translate: impl IntoMaybeDynCowStr,
        /// An enumerated attribute used to control the on-screen virtual keyboard behavior on devices such as tablets, mobile phones, or other devices where a hardware keyboard may not be available for elements that its content is editable (for example, it is an `<input>` or `<textarea>` element, or an element with the `contenteditable` attribute set).
        /// `auto` or an _empty string_, which automatically shows the virtual keyboard when the element is focused or tapped.
        /// `manual`, which decouples focus and tap on the element from the virtual keyboard's state.
        virtualkeyboardpolicy: impl IntoMaybeDynCowStr,
    }
}

/// A trait that is implemented for all SVG elements and which provides all the global SVG
/// attributes.
///
/// Reference: <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute>
pub trait SvgGlobalAttributes: SetAttribute + Sized {
    impl_attributes! {
        accentHeight("accent-height"): impl IntoMaybeDynCowStr,
        accumulate: impl IntoMaybeDynCowStr,
        additive: impl IntoMaybeDynCowStr,
        alignmentBaseline("alignment-baseline"): impl IntoMaybeDynCowStr,
        alphabetic: impl IntoMaybeDynCowStr,
        amplitude: impl IntoMaybeDynCowStr,
        arabicForm("arabic-form"): impl IntoMaybeDynCowStr,
        ascent: impl IntoMaybeDynCowStr,
        attributeName("attributeName"): impl IntoMaybeDynCowStr,
        attributeType("attributeType"): impl IntoMaybeDynCowStr,
        azimuth: impl IntoMaybeDynCowStr,
        baseFrequency("baseFrequency"): impl IntoMaybeDynCowStr,
        baselineShift("baseline-shift"): impl IntoMaybeDynCowStr,
        baseProfile("baseProfile"): impl IntoMaybeDynCowStr,
        bbox: impl IntoMaybeDynCowStr,
        begin: impl IntoMaybeDynCowStr,
        bias: impl IntoMaybeDynCowStr,
        by: impl IntoMaybeDynCowStr,
        calcMode("calcMode"): impl IntoMaybeDynCowStr,
        capHeight("cap-height"): impl IntoMaybeDynCowStr,
        class: impl IntoMaybeDynCowStr,
        clipPathUnits("clipPathUnits"): impl IntoMaybeDynCowStr,
        clipPath("clip-path"): impl IntoMaybeDynCowStr,
        clipRule("clip-rule"): impl IntoMaybeDynCowStr,
        color: impl IntoMaybeDynCowStr,
        colorInterpolation("color-interpolation"): impl IntoMaybeDynCowStr,
        colorInterpolationFilters("color-interpolation-filters"): impl IntoMaybeDynCowStr,
        colorProfile("color-profile"): impl IntoMaybeDynCowStr,
        colorRendering("color-rendering"): impl IntoMaybeDynCowStr,
        crossorigin: impl IntoMaybeDynCowStr,
        cursor: impl IntoMaybeDynCowStr,
        cx: impl IntoMaybeDynCowStr,
        cy: impl IntoMaybeDynCowStr,
        d: impl IntoMaybeDynCowStr,
        decelerate: impl IntoMaybeDynCowStr,
        descent: impl IntoMaybeDynCowStr,
        diffuseConstant("diffuseConstant"): impl IntoMaybeDynCowStr,
        direction: impl IntoMaybeDynCowStr,
        display: impl IntoMaybeDynCowStr,
        divisor: impl IntoMaybeDynCowStr,
        dominantBaseline("dominant-baseline"): impl IntoMaybeDynCowStr,
        dur: impl IntoMaybeDynCowStr,
        dx: impl IntoMaybeDynCowStr,
        dy: impl IntoMaybeDynCowStr,
        edgeMode("edgeMode"): impl IntoMaybeDynCowStr,
        elevation: impl IntoMaybeDynCowStr,
        enableBackground("enable-background"): impl IntoMaybeDynCowStr,
        end: impl IntoMaybeDynCowStr,
        exponent: impl IntoMaybeDynCowStr,
        fill: impl IntoMaybeDynCowStr,
        fillOpacity("fill-opacity"): impl IntoMaybeDynCowStr,
        fillRule("fill-rule"): impl IntoMaybeDynCowStr,
        filter: impl IntoMaybeDynCowStr,
        filterUnits("filterUnits"): impl IntoMaybeDynCowStr,
        floodColor("flood-color"): impl IntoMaybeDynCowStr,
        floodOpacity("flood-opacity"): impl IntoMaybeDynCowStr,
        fontFamily("font-family"): impl IntoMaybeDynCowStr,
        fontSize("font-size"): impl IntoMaybeDynCowStr,
        fontSizeAdjust("font-size-adjust"): impl IntoMaybeDynCowStr,
        fontStretch("font-stretch"): impl IntoMaybeDynCowStr,
        fontStyle("font-style"): impl IntoMaybeDynCowStr,
        fontVariant("font-variant"): impl IntoMaybeDynCowStr,
        fontWeight("font-weight"): impl IntoMaybeDynCowStr,
        format: impl IntoMaybeDynCowStr,
        from: impl IntoMaybeDynCowStr,
        fr: impl IntoMaybeDynCowStr,
        fx: impl IntoMaybeDynCowStr,
        fy: impl IntoMaybeDynCowStr,
        g1: impl IntoMaybeDynCowStr,
        g2: impl IntoMaybeDynCowStr,
        glyphName("glyph-name"): impl IntoMaybeDynCowStr,
        glyphOrientationHorizontal("glyph-orientation-horizontal"): impl IntoMaybeDynCowStr,
        glyphOrientationVertical("glyph-orientation-vertical"): impl IntoMaybeDynCowStr,
        glyphRef: impl IntoMaybeDynCowStr,
        gradientTransform("gradientTransform"): impl IntoMaybeDynCowStr,
        gradientUnits("gradientUnits"): impl IntoMaybeDynCowStr,
        hanging: impl IntoMaybeDynCowStr,
        height: impl IntoMaybeDynCowStr,
        href: impl IntoMaybeDynCowStr,
        hreflang: impl IntoMaybeDynCowStr,
        horizAdvX("horiz-adv-x"): impl IntoMaybeDynCowStr,
        horizOriginX("horiz-origin-x"): impl IntoMaybeDynCowStr,
        id: impl IntoMaybeDynCowStr,
        ideographic: impl IntoMaybeDynCowStr,
        imageRendering("image-rendering"): impl IntoMaybeDynCowStr,
        in_: impl IntoMaybeDynCowStr,
        in2: impl IntoMaybeDynCowStr,
        intercept: impl IntoMaybeDynCowStr,
        k: impl IntoMaybeDynCowStr,
        k1: impl IntoMaybeDynCowStr,
        k2: impl IntoMaybeDynCowStr,
        k3: impl IntoMaybeDynCowStr,
        k4: impl IntoMaybeDynCowStr,
        kernelMatrix("kernelMatrix"): impl IntoMaybeDynCowStr,
        kernelUnitLength("kernelUnitLength"): impl IntoMaybeDynCowStr,
        kerning: impl IntoMaybeDynCowStr,
        keyPoints("keyPoints"): impl IntoMaybeDynCowStr,
        keySplines("keySplines"): impl IntoMaybeDynCowStr,
        keyTimes("keyTimes"): impl IntoMaybeDynCowStr,
        lang: impl IntoMaybeDynCowStr,
        lengthAdjust("lengthAdjust"): impl IntoMaybeDynCowStr,
        letterSpacing("letter-spacing"): impl IntoMaybeDynCowStr,
        lightingColor("lighting-color"): impl IntoMaybeDynCowStr,
        limitingConeAngle("limitingConeAngle"): impl IntoMaybeDynCowStr,
        local: impl IntoMaybeDynCowStr,
        markerEnd("marker-end"): impl IntoMaybeDynCowStr,
        markerMid("marker-mid"): impl IntoMaybeDynCowStr,
        markerStart("marker-start"): impl IntoMaybeDynCowStr,
        markerHeight("markerHeight"): impl IntoMaybeDynCowStr,
        markerUnits("markerUnits"): impl IntoMaybeDynCowStr,
        markerWidth("markerWidth"): impl IntoMaybeDynCowStr,
        mask: impl IntoMaybeDynCowStr,
        maskContentUnits("maskContentUnits"): impl IntoMaybeDynCowStr,
        maskUnits("maskUnits"): impl IntoMaybeDynCowStr,
        mathematical: impl IntoMaybeDynCowStr,
        max: impl IntoMaybeDynCowStr,
        media: impl IntoMaybeDynCowStr,
        method: impl IntoMaybeDynCowStr,
        min: impl IntoMaybeDynCowStr,
        mode: impl IntoMaybeDynCowStr,
        name: impl IntoMaybeDynCowStr,
        numOctaves("numOctaves"): impl IntoMaybeDynCowStr,
        offset: impl IntoMaybeDynCowStr,
        opacity: impl IntoMaybeDynCowStr,
        operator: impl IntoMaybeDynCowStr,
        order: impl IntoMaybeDynCowStr,
        orient: impl IntoMaybeDynCowStr,
        orientation: impl IntoMaybeDynCowStr,
        origin: impl IntoMaybeDynCowStr,
        overflow: impl IntoMaybeDynCowStr,
        overlinePosition("overline-position"): impl IntoMaybeDynCowStr,
        overlineThickness("overline-thickness"): impl IntoMaybeDynCowStr,
        panose1("panose-1"): impl IntoMaybeDynCowStr,
        paintOrder("paint-order"): impl IntoMaybeDynCowStr,
        path: impl IntoMaybeDynCowStr,
        pathLength("pathLength"): impl IntoMaybeDynCowStr,
        patternContentUnits("patternContentUnits"): impl IntoMaybeDynCowStr,
        patternTransform("patternTransform"): impl IntoMaybeDynCowStr,
        patternUnits("patternUnits"): impl IntoMaybeDynCowStr,
        ping: impl IntoMaybeDynCowStr,
        pointerEvents("pointer-events"): impl IntoMaybeDynCowStr,
        points: impl IntoMaybeDynCowStr,
        pointsAtX("pointsAtX"): impl IntoMaybeDynCowStr,
        pointsAtY("pointsAtY"): impl IntoMaybeDynCowStr,
        pointsAtZ("pointsAtZ"): impl IntoMaybeDynCowStr,
        preserveAlpha("preserveAlpha"): impl IntoMaybeDynCowStr,
        preserveAspectRatio("preserveAspectRatio"): impl IntoMaybeDynCowStr,
        primitiveUnits("primitiveUnits"): impl IntoMaybeDynCowStr,
        r: impl IntoMaybeDynCowStr,
        radius: impl IntoMaybeDynCowStr,
        referrerPolicy("referrerPolicy"): impl IntoMaybeDynCowStr,
        refX("refX"): impl IntoMaybeDynCowStr,
        refY("refY"): impl IntoMaybeDynCowStr,
        rel: impl IntoMaybeDynCowStr,
        renderingIntent("rendering-intent"): impl IntoMaybeDynCowStr,
        repeatCount("repeatCount"): impl IntoMaybeDynCowStr,
        repeatDur("repeatDur"): impl IntoMaybeDynCowStr,
        requiredExtensions("requiredExtensions"): impl IntoMaybeDynCowStr,
        requiredFeatures("requiredFeatures"): impl IntoMaybeDynCowStr,
        restart: impl IntoMaybeDynCowStr,
        result: impl IntoMaybeDynCowStr,
        rotate: impl IntoMaybeDynCowStr,
        rx: impl IntoMaybeDynCowStr,
        ry: impl IntoMaybeDynCowStr,
        scale: impl IntoMaybeDynCowStr,
        seed: impl IntoMaybeDynCowStr,
        shapeRendering("shape-rendering"): impl IntoMaybeDynCowStr,
        slope: impl IntoMaybeDynCowStr,
        spacing: impl IntoMaybeDynCowStr,
        specularConstant("specularConstant"): impl IntoMaybeDynCowStr,
        specularExponent("specularExponent"): impl IntoMaybeDynCowStr,
        speed: impl IntoMaybeDynCowStr,
        spreadMethod("spreadMethod"): impl IntoMaybeDynCowStr,
        startOffset("startOffset"): impl IntoMaybeDynCowStr,
        stdDeviation("stdDeviation"): impl IntoMaybeDynCowStr,
        stemh: impl IntoMaybeDynCowStr,
        stemv: impl IntoMaybeDynCowStr,
        stitchTiles("stitchTiles"): impl IntoMaybeDynCowStr,
        stopColor("stop-color"): impl IntoMaybeDynCowStr,
        stopOpacity("stop-opacity"): impl IntoMaybeDynCowStr,
        strikethroughPosition("strikethrough-position"): impl IntoMaybeDynCowStr,
        strikethroughThickness("strikethrough-thickness"): impl IntoMaybeDynCowStr,
        string: impl IntoMaybeDynCowStr,
        stroke: impl IntoMaybeDynCowStr,
        strokeDasharray("stroke-dasharray"): impl IntoMaybeDynCowStr,
        strokeDashoffset("stroke-dashoffset"): impl IntoMaybeDynCowStr,
        strokeLinecap("stroke-linecap"): impl IntoMaybeDynCowStr,
        strokeLinejoin("stroke-linejoin"): impl IntoMaybeDynCowStr,
        strokeMiterlimit("stroke-miterlimit"): impl IntoMaybeDynCowStr,
        strokeOpacity("stroke-opacity"): impl IntoMaybeDynCowStr,
        strokeWidth("stroke-width"): impl IntoMaybeDynCowStr,
        style: impl IntoMaybeDynCowStr,
        surfaceScale("surfaceScale"): impl IntoMaybeDynCowStr,
        systemLanguage("systemLanguage"): impl IntoMaybeDynCowStr,
        tabindex: impl IntoMaybeDynCowStr,
        tableValues("tableValues"): impl IntoMaybeDynCowStr,
        target: impl IntoMaybeDynCowStr,
        targetX("targetX"): impl IntoMaybeDynCowStr,
        targetY("targetY"): impl IntoMaybeDynCowStr,
        textAnchor("text-anchor"): impl IntoMaybeDynCowStr,
        textDecoration("text-decoration"): impl IntoMaybeDynCowStr,
        textRendering("text-rendering"): impl IntoMaybeDynCowStr,
        textLength("textLength"): impl IntoMaybeDynCowStr,
        to: impl IntoMaybeDynCowStr,
        transform: impl IntoMaybeDynCowStr,
        transformOrigin("transform-origin"): impl IntoMaybeDynCowStr,
        type_: impl IntoMaybeDynCowStr,
        u1: impl IntoMaybeDynCowStr,
        u2: impl IntoMaybeDynCowStr,
        underlinePosition("underline-position"): impl IntoMaybeDynCowStr,
        underlineThickness("underline-thickness"): impl IntoMaybeDynCowStr,
        unicode: impl IntoMaybeDynCowStr,
        unicodeBidi("unicode-bidi"): impl IntoMaybeDynCowStr,
        unicodeRange("unicode-range"): impl IntoMaybeDynCowStr,
        unitsPerEm("units-per-em"): impl IntoMaybeDynCowStr,
        vAlphabetic("v-alphabetic"): impl IntoMaybeDynCowStr,
        vHanging("v-hanging"): impl IntoMaybeDynCowStr,
        vIdeographic("v-ideographic"): impl IntoMaybeDynCowStr,
        vMathematical("v-mathematical"): impl IntoMaybeDynCowStr,
        values: impl IntoMaybeDynCowStr,
        vectorEffect("vector-effect"): impl IntoMaybeDynCowStr,
        version: impl IntoMaybeDynCowStr,
        vertAdvY("vert-adv-y"): impl IntoMaybeDynCowStr,
        vertOriginX("vert-origin-x"): impl IntoMaybeDynCowStr,
        vertOriginY("vert-origin-y"): impl IntoMaybeDynCowStr,
        viewBox: impl IntoMaybeDynCowStr,
        visibility: impl IntoMaybeDynCowStr,
        width: impl IntoMaybeDynCowStr,
        widths: impl IntoMaybeDynCowStr,
        wordSpacing("word-spacing"): impl IntoMaybeDynCowStr,
        writingMode("writing-mode"): impl IntoMaybeDynCowStr,
        x: impl IntoMaybeDynCowStr,
        xHeight("x-height"): impl IntoMaybeDynCowStr,
        x1: impl IntoMaybeDynCowStr,
        x2: impl IntoMaybeDynCowStr,
        xChannelSelector("xChannelSelector"): impl IntoMaybeDynCowStr,
        xmlBase("xml:base"): impl IntoMaybeDynCowStr,
        xmlLang("xml:lang"): impl IntoMaybeDynCowStr,
        xmlSpace("xml:space"): impl IntoMaybeDynCowStr,
        y: impl IntoMaybeDynCowStr,
        y1: impl IntoMaybeDynCowStr,
        y2: impl IntoMaybeDynCowStr,
        yChannelSelector("yChannelSelector"): impl IntoMaybeDynCowStr,
        zoomAndPan("zoomAndPan"): impl IntoMaybeDynCowStr,
    }
}

/// Attributes that are available on all elements.
pub trait GlobalAttributes: SetAttribute + Sized {
    /// Set attribute `name` with `value`.
    fn attr(mut self, name: &'static str, value: impl IntoMaybeDynCowStr) -> Self {
        self.set_attribute(name, value.into_maybe_dyn());
        self
    }

    /// Set attribute `name` with `value`.
    fn bool_attr(mut self, name: &'static str, value: impl IntoMaybeDynBool) -> Self {
        self.set_attribute(name, value.into_maybe_dyn());
        self
    }

    /// Set JS property `name` with `value`.
    fn prop(mut self, name: &'static str, value: impl IntoMaybeDynJsValue) -> Self {
        self.set_attribute(name, value.into_maybe_dyn());
        self
    }

    /// Set an event handler with `name`.
    fn on<E: events::EventDescriptor, R>(
        mut self,
        _: E,
        mut handler: impl EventHandler<E, R>,
    ) -> Self {
        let scope = use_current_scope(); // Run handler inside the current scope.
        let handler = move |ev: web_sys::Event| scope.run_in(|| handler.call(ev.unchecked_into()));
        self.set_event_handler(E::NAME, handler);
        self
    }

    /// Set a two way binding with `name`.
    fn bind<E: bind::BindDescriptor>(mut self, _: E, signal: Signal<E::ValueTy>) -> Self {
        let scope = use_current_scope(); // Run handler inside the current scope.
        let handler = move |ev: web_sys::Event| {
            scope.run_in(|| {
                let value =
                    js_sys::Reflect::get(&ev.current_target().unwrap(), &E::TARGET_PROPERTY.into())
                        .unwrap();
                signal.set(E::CONVERT_FROM_JS(&value).expect("failed to convert value from js"));
            })
        };
        self.set_event_handler(<E::Event as events::EventDescriptor>::NAME, handler);

        self.prop(E::TARGET_PROPERTY, move || signal.get_clone().into())
    }
}

impl<T: GlobalProps> GlobalAttributes for T {}

/// Props that are available on all elements.
pub trait GlobalProps: GlobalAttributes + AsHtmlNode + Sized {
    /// Set the inner html of an element.
    fn dangerously_set_inner_html(mut self, inner_html: impl Into<Cow<'static, str>>) -> Self {
        self.as_html_node().set_inner_html(inner_html.into());
        self
    }

    /// Set the children of an element.
    fn children(mut self, children: impl Into<View>) -> Self {
        self.as_html_node().append_view(children.into());
        self
    }

    /// Set a [`NodeRef`] on this element.
    fn r#ref(mut self, noderef: NodeRef) -> Self {
        if is_not_ssr!() {
            noderef.set(Some(self.as_html_node().as_web_sys().clone()));
        }
        self
    }

    fn spread(mut self, attributes: Attributes) -> Self {
        attributes.apply_self(self.as_html_node());
        self
    }
}

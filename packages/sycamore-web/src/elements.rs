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
            self.set_attribute($name, value.into());
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
            download: impl Into<MaybeDyn<Cow<'static, str>>>,
            href: impl Into<MaybeDyn<Cow<'static, str>>>,
            hreflang: impl Into<MaybeDyn<Cow<'static, str>>>,
            target: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
            ping: impl Into<MaybeDyn<Cow<'static, str>>>,
            rel: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        abbr {},
        address {},
        area {
            alt: impl Into<MaybeDyn<Cow<'static, str>>>,
            coords: impl Into<MaybeDyn<Cow<'static, str>>>,
            download: impl Into<MaybeDyn<Cow<'static, str>>>,
            href: impl Into<MaybeDyn<Cow<'static, str>>>,
            hreflang: impl Into<MaybeDyn<Cow<'static, str>>>,
            media: impl Into<MaybeDyn<Cow<'static, str>>>,
            referrerpolicy: impl Into<MaybeDyn<Cow<'static, str>>>,
            ping: impl Into<MaybeDyn<Cow<'static, str>>>,
            rel: impl Into<MaybeDyn<Cow<'static, str>>>,
            shape: impl Into<MaybeDyn<Cow<'static, str>>>,
            target: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        article {},
        aside {},
        audio {
            autoplay: impl Into<MaybeDyn<bool>>,
            controls: impl Into<MaybeDyn<bool>>,
            crossorigin: impl Into<MaybeDyn<Cow<'static, str>>>,
            muted: impl Into<MaybeDyn<bool>>,
            preload: impl Into<MaybeDyn<Cow<'static, str>>>,
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#loop("loop"): impl Into<MaybeDyn<bool>>,
        },
        b {},
        base {
            href: impl Into<MaybeDyn<Cow<'static, str>>>,
            target: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        bdi {},
        bdo {},
        blockquote {
            cite: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        body {},
        br {},
        /// The `<button>` HTML element represents a clickable button, used to submit forms or anywhere in a document for accessible, standard button functionality.
        ///
        /// By default, HTML buttons are presented in a style resembling the platform the user agent runs on, but you can change buttons’ appearance with CSS.
        button {
            autofocus: impl Into<MaybeDyn<bool>>,
            disabled: impl Into<MaybeDyn<bool>>,
            form: impl Into<MaybeDyn<Cow<'static, str>>>,
            formaction: impl Into<MaybeDyn<Cow<'static, str>>>,
            formenctype: impl Into<MaybeDyn<Cow<'static, str>>>,
            formmethod: impl Into<MaybeDyn<Cow<'static, str>>>,
            formnovalidate: impl Into<MaybeDyn<bool>>,
            formtarget: impl Into<MaybeDyn<Cow<'static, str>>>,
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
            value: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        canvas {
            height: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            width: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
        },
        caption {},
        cite {},
        code {
            language: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        col {
            span: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
        },
        colgroup {
            span: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
        },
        data {
            value: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        datalist {},
        dd {},
        del {
            cite: impl Into<MaybeDyn<Cow<'static, str>>>,
            datetime: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        details {
            open: impl Into<MaybeDyn<bool>>,
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
            height: impl Into<MaybeDyn<Cow<'static, str>>>,
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
            width: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
        },
        fieldset {},
        figcaption {},
        figure {},
        footer {},
        form {
            acceptcharset: impl Into<MaybeDyn<Cow<'static, str>>>,
            action: impl Into<MaybeDyn<Cow<'static, str>>>,
            autocomplete: impl Into<MaybeDyn<Cow<'static, str>>>,
            enctype: impl Into<MaybeDyn<Cow<'static, str>>>,
            method: impl Into<MaybeDyn<Cow<'static, str>>>,
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
            novalidate: impl Into<MaybeDyn<bool>>,
            target: impl Into<MaybeDyn<Cow<'static, str>>>,
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
            allow: impl Into<MaybeDyn<Cow<'static, str>>>,
            allowfullscreen: impl Into<MaybeDyn<bool>>,
            allowpaymentrequest: impl Into<MaybeDyn<bool>>,
            height: impl Into<MaybeDyn<Cow<'static, str>>>,
            loading: impl Into<MaybeDyn<Cow<'static, str>>>,
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
            referrerpolicy: impl Into<MaybeDyn<Cow<'static, str>>>,
            sandbox: impl Into<MaybeDyn<bool>>,
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            srcdoc: impl Into<MaybeDyn<Cow<'static, str>>>,
            width: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        img {
            alt: impl Into<MaybeDyn<Cow<'static, str>>>,
            crossorigin: impl Into<MaybeDyn<Cow<'static, str>>>,
            decoding: impl Into<MaybeDyn<Cow<'static, str>>>,
            height: impl Into<MaybeDyn<Cow<'static, str>>>,
            ismap: impl Into<MaybeDyn<bool>>,
            loading: impl Into<MaybeDyn<Cow<'static, str>>>,
            referrerpolicy: impl Into<MaybeDyn<Cow<'static, str>>>,
            sizes: impl Into<MaybeDyn<Cow<'static, str>>>,
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            srcset: impl Into<MaybeDyn<Cow<'static, str>>>,
            usemap: impl Into<MaybeDyn<Cow<'static, str>>>,
            width: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        /// The `<input>` HTML element is used to create interactive controls for web-based forms in order to accept data from the user; a wide variety of types of input data and control widgets are available, depending on the device and user agent. The `<input>` element is one of the most powerful and complex in all of HTML due to the sheer number of combinations of input types and attributes.
        input {
            accept: impl Into<MaybeDyn<Cow<'static, str>>>,
            alt: impl Into<MaybeDyn<Cow<'static, str>>>,
            autocomplete: impl Into<MaybeDyn<Cow<'static, str>>>,
            autofocus: impl Into<MaybeDyn<bool>>,
            capture: impl Into<MaybeDyn<Cow<'static, str>>>,
            checked: impl Into<MaybeDyn<bool>>,
            directory: impl Into<MaybeDyn<Cow<'static, str>>>,
            disabled: impl Into<MaybeDyn<bool>>,
            form: impl Into<MaybeDyn<Cow<'static, str>>>,
            formaction: impl Into<MaybeDyn<Cow<'static, str>>>,
            formenctype: impl Into<MaybeDyn<Cow<'static, str>>>,
            formmethod: impl Into<MaybeDyn<Cow<'static, str>>>,
            formnovalidate: impl Into<MaybeDyn<bool>>,
            formtarget: impl Into<MaybeDyn<Cow<'static, str>>>,
            height: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            initial_checked: impl Into<MaybeDyn<bool>>,
            initial_value: impl Into<MaybeDyn<Cow<'static, str>>>,
            list: impl Into<MaybeDyn<Cow<'static, str>>>,
            max: impl Into<MaybeDyn<Cow<'static, str>>>,
            maxlength: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            min: impl Into<MaybeDyn<Cow<'static, str>>>,
            minlength: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            multiple: impl Into<MaybeDyn<bool>>,
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
            pattern: impl Into<MaybeDyn<Cow<'static, str>>>,
            placeholder: impl Into<MaybeDyn<Cow<'static, str>>>,
            readonly: impl Into<MaybeDyn<bool>>,
            required: impl Into<MaybeDyn<bool>>,
            size: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            spellcheck: impl Into<MaybeDyn<bool>>,
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            step: impl Into<MaybeDyn<Cow<'static, str>>>,
            tabindex: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
            value: impl Into<MaybeDyn<Cow<'static, str>>>,
            width: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
        },
        ins {
            cite: impl Into<MaybeDyn<Cow<'static, str>>>,
            datetime: impl Into<MaybeDyn<Cow<'static, str>>>,
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
            form: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#for("for"): impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        legend {},
        /// The `<li>` HTML element is used to represent an item in a list. It must be contained in a parent element: an ordered list (`<ol>`), an unordered list (`<ul>`), or a menu (`<menu>`). In menus and unordered lists, list items are usually displayed using bullet points. In ordered lists, they are usually displayed with an ascending counter on the left, such as a number or letter.
        li {
            value: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
        },
        link {
            r#as("as"): impl Into<MaybeDyn<Cow<'static, str>>>,
            crossorigin: impl Into<MaybeDyn<Cow<'static, str>>>,
            href: impl Into<MaybeDyn<Cow<'static, str>>>,
            hreflang: impl Into<MaybeDyn<Cow<'static, str>>>,
            media: impl Into<MaybeDyn<Cow<'static, str>>>,
            rel: impl Into<MaybeDyn<Cow<'static, str>>>,
            sizes: impl Into<MaybeDyn<Cow<'static, str>>>,
            title: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
            integrity: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        main {},
        map {
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        mark {},
        menu {},
        menuitem {},
        meta {
            charset: impl Into<MaybeDyn<Cow<'static, str>>>,
            content: impl Into<MaybeDyn<Cow<'static, str>>>,
            http_equiv("http-equiv"): impl Into<MaybeDyn<Cow<'static, str>>>,
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        meter {
            value: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            min: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            max: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            low: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            high: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            optimum: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            form: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        nav {},
        noscript {},
        object {
            data: impl Into<MaybeDyn<Cow<'static, str>>>,
            form: impl Into<MaybeDyn<Cow<'static, str>>>,
            height: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
            typemustmatch: impl Into<MaybeDyn<bool>>,
            usemap: impl Into<MaybeDyn<Cow<'static, str>>>,
            width: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        /// The `<ol>` HTML element represents an ordered list of items — typically rendered as a numbered list.
        ol {
            reversed: impl Into<MaybeDyn<bool>>,
            start: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        optgroup {
            disabled: impl Into<MaybeDyn<bool>>,
            label: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        option {
            disabled: impl Into<MaybeDyn<bool>>,
            initial_selected: impl Into<MaybeDyn<bool>>,
            label: impl Into<MaybeDyn<Cow<'static, str>>>,
            selected: impl Into<MaybeDyn<bool>>,
            value: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        output {
            r#for("for"): impl Into<MaybeDyn<Cow<'static, str>>>,
            form: impl Into<MaybeDyn<Cow<'static, str>>>,
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        /// The `<p>` HTML element represents a paragraph. Paragraphs are usually represented in visual media as blocks of text separated from adjacent blocks by blank lines and/or first-line indentation, but HTML paragraphs can be any structural grouping of related content, such as images or form fields.
        ///
        /// Paragraphs are block-level elements, and notably will automatically close if another block-level element is parsed before the closing `</p>` tag.
        p {},
        param {
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
            value: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        picture {},
        pre {},
        progress {
            value: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: f64 value
            max: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: f64 value
        },
        q {
            cite: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        rp {},
        rt {},
        ruby {},
        s {},
        samp {},
        script {
            r#async: impl Into<MaybeDyn<bool>>,
            crossorigin: impl Into<MaybeDyn<Cow<'static, str>>>,
            defer: impl Into<MaybeDyn<bool>>,
            integrity: impl Into<MaybeDyn<Cow<'static, str>>>,
            nomodule: impl Into<MaybeDyn<bool>>,
            nonce: impl Into<MaybeDyn<Cow<'static, str>>>,
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            script: impl Into<MaybeDyn<Cow<'static, str>>>,
            text: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        section {},
        select {
            autocomplete: impl Into<MaybeDyn<Cow<'static, str>>>,
            autofocus: impl Into<MaybeDyn<bool>>,
            disabled: impl Into<MaybeDyn<bool>>,
            form: impl Into<MaybeDyn<Cow<'static, str>>>,
            multiple: impl Into<MaybeDyn<bool>>,
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
            required: impl Into<MaybeDyn<bool>>,
            size: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            value: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        small {},
        source {
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        /// The `<span>` HTML element is a generic inline container for phrasing content, which does not inherently represent anything. It can be used to group elements for styling purposes (using the class or id attributes), or because they share attribute values, such as lang. It should be used only when no other semantic element is appropriate. `<span>` is very much like a `<div>` element, but `<div>` is a block-level element whereas a `<span>` is an inline element.
        span {},
        strong {},
        style {
            media: impl Into<MaybeDyn<Cow<'static, str>>>,
            nonce: impl Into<MaybeDyn<Cow<'static, str>>>,
            title: impl Into<MaybeDyn<Cow<'static, str>>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        sub {},
        summary {},
        sup {},
        table {},
        tbody {},
        td {
            colspan: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            headers: impl Into<MaybeDyn<Cow<'static, str>>>,
            rowspan: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
        },
        template {},
        textarea {
            autocomplete: impl Into<MaybeDyn<Cow<'static, str>>>,
            autofocus: impl Into<MaybeDyn<bool>>,
            cols: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            disabled: impl Into<MaybeDyn<bool>>,
            form: impl Into<MaybeDyn<Cow<'static, str>>>,
            initial_value: impl Into<MaybeDyn<Cow<'static, str>>>,
            maxlength: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            minlength: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            name: impl Into<MaybeDyn<Cow<'static, str>>>,
            placeholder: impl Into<MaybeDyn<Cow<'static, str>>>,
            readonly: impl Into<MaybeDyn<bool>>,
            required: impl Into<MaybeDyn<bool>>,
            rows: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            spellcheck: impl Into<MaybeDyn<bool>>,
            r#type("type"): impl Into<MaybeDyn<Cow<'static, str>>>,
            value: impl Into<MaybeDyn<Cow<'static, str>>>,
            wrap: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        tfoot {},
        th {
            abbr: impl Into<MaybeDyn<Cow<'static, str>>>,
            colspan: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            headers: impl Into<MaybeDyn<Cow<'static, str>>>,
            rowspan: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            scope: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        thead {},
        time {
            datetime: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        title {},
        tr {},
        track {
            default: impl Into<MaybeDyn<bool>>,
            kind: impl Into<MaybeDyn<Cow<'static, str>>>,
            label: impl Into<MaybeDyn<Cow<'static, str>>>,
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            srclang: impl Into<MaybeDyn<Cow<'static, str>>>,
        },
        u {},
        /// The `<ul>` HTML element represents an unordered list of items, typically rendered as a bulleted list.
        ul {},
        var {},
        video {
            autoplay: impl Into<MaybeDyn<bool>>,
            controls: impl Into<MaybeDyn<bool>>,
            crossorigin: impl Into<MaybeDyn<Cow<'static, str>>>,
            height: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
            r#loop("loop"): impl Into<MaybeDyn<bool>>,
            muted: impl Into<MaybeDyn<bool>>,
            playsinline: impl Into<MaybeDyn<bool>>,
            poster: impl Into<MaybeDyn<Cow<'static, str>>>,
            preload: impl Into<MaybeDyn<Cow<'static, str>>>,
            src: impl Into<MaybeDyn<Cow<'static, str>>>,
            width: impl Into<MaybeDyn<Cow<'static, str>>>, // TODO: int value
        },
        wbr {},
    }

    impl_svg_elements! {
        svg {
            xmlns: impl Into<MaybeDyn<Cow<'static, str>>>,
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
        accesskey: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Controls whether inputted text is automatically capitalized and, if so, in what manner.
        autocapitalize: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Indicates that an element is to be focused on page load, or as soon as the `<dialog>` it is part of is displayed. This attribute is a boolean, initially false.
        autofocus: impl Into<MaybeDyn<bool>>,
        /// The class global attribute is a space-separated list of the case-sensitive classes of the element.
        /// Classes allow CSS and JavaScript to select and access specific elements via the class selectors.
        class: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// An enumerated attribute indicating if the element should be editable by the user. If so, the browser modifies its widget to allow editing. The attribute must take one of the following values:
        /// * `true` or the empty string, which indicates that the element must be editable;
        /// * `false`, which indicates that the element must not be editable.
        contenteditable: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// An enumerated attribute indicating the directionality of the element's text. It can have the following values:
        /// * `ltr`, which means left to right and is to be used for languages that are written from the left to the right (like English);
        /// * `rtl`, which means right to left and is to be used for languages that are written from the right to the left (like Arabic);
        /// * `auto`, which lets the user agent decide. It uses a basic algorithm as it parses the characters inside the element until it finds a character with a strong directionality, then it applies that directionality to the whole element.
        dir: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// An enumerated attribute indicating whether the element can be dragged, using the Drag and Drop API. It can have the following values:
        /// * `true`, which indicates that the element may be dragged
        /// * `false`, which indicates that the element may not be dragged.
        draggable: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Hints what action label (or icon) to present for the enter key on virtual keyboards.
        enterkeyhint: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Used to transitively export shadow parts from a nested shadow tree into a containing light tree.
        exportparts: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// An enumerated attribute indicating that the element is not yet, or is no longer, _relevant_. For example, it can be used to hide elements of the page that can't be used until the login process has been completed. The browser won't render such elements. This attribute must not be used to hide content that could legitimately be shown.
        hidden: impl Into<MaybeDyn<bool>>,
        /// The id global attribute defines an identifier (ID) which must be unique in the whole document. Its purpose is to identify the element when linking (using a fragment identifier), scripting, or styling (with CSS).
        id: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// A boolean value that makes the browser disregard user input events for the element. Useful when click events are present.
        inert: impl Into<MaybeDyn<bool>>,
        /// Provides a hint to browsers about the type of virtual keyboard configuration to use when editing this element or its contents. Used primarily on `<input>` elements, but is usable on any element while in contenteditable mode.
        inputmode: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// The is global attribute allows you to specify that a standard HTML element should behave like a defined custom built-in element.
        ///
        /// This attribute can only be used if the specified custom element name has been successfully defined in the current document, and extends the element type it is being applied to.
        is: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// The unique, global identifier of an item.
        itemid: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Used to add properties to an item. Every HTML element may have an `itemprop` attribute specified, where an `itemprop` consists of a name and value pair.
        itemprop: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Properties that are not descendants of an element with the `itemscope` attribute can be associated with the item using an `itemref`. It provides a list of element ids (not `itemid`s) with additional properties elsewhere in the document.
        itemref: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// `itemscope` (usually) works along with `itemtype` to specify that the HTML contained in a block is about a particular item. `itemscope` creates the Item and defines the scope of the `itemtype` associated with it. `itemtype` is a valid URL of a vocabulary (such as schema.org) that describes the item and its properties context.
        itemscope: impl Into<MaybeDyn<bool>>,
        /// Specifies the URL of the vocabulary that will be used to define `itemprops` (item properties) in the data structure. `itemscope` is used to set the scope of where in the data structure the vocabulary set by `itemtype` will be active.
        itemtype: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Helps define the language of an element: the language that non-editable elements are in, or the language that editable elements should be written in by the user. The attribute contains one "language tag" (made of hyphen-separated "language subtags") in the format defined in [RFC 5646: Tags for Identifying Languages (also known as BCP 47)](https://datatracker.ietf.org/doc/html/rfc5646). `xml:lang` has priority over it.
        lang: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// A cryptographic nonce ("number used once") which can be used by Content Security Policy to determine whether or not a given fetch will be allowed to proceed.
        nonce: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// A space-separated list of the part names of the element. Part names allows CSS to select and style specific elements in a shadow tree via the `::part` pseudo-element.
        part: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Used to designate an element as a popover element (see Popover API). Popover elements are hidden via `display: none` until opened via an invoking/control element (i.e. a `<button>` or `<input type="button">` with a popovertarget attribute) or a `HTMLElement.showPopover()` call.
        popover: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Roles define the semantic meaning of content, allowing screen readers and other tools to present and support interaction with an object in a way that is consistent with user expectations of that type of object. `roles` are added to HTML elements using `role="role_type"`, where `role_type` is the name of a role in the ARIA specification.
        role: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// The slot global attribute assigns a slot in a shadow DOM shadow tree to an element: An element with a slot attribute is assigned to the slot created by the `<slot>` element whose name attribute's value matches that slot attribute's value.
        slot: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// An enumerated attribute defines whether the element may be checked for spelling errors. It may have the following values:
        /// * empty string or `true`, which indicates that the element should be, if possible, checked for spelling errors;
        /// * `false`, which indicates that the element should not be checked for spelling errors.
        spellcheck: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Contains CSS styling declarations to be applied to the element. Note that it is recommended for styles to be defined in a separate file or files. This attribute and the `<style>` element have mainly the purpose of allowing for quick styling, for example for testing purposes.
        style: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// An integer attribute indicating if the element can take input focus (is focusable), if it should participate to sequential keyboard navigation, and if so, at what position. It can take several values:
        /// * a _negative value_ means that the element should be focusable, but should not be reachable via sequential keyboard navigation;
        /// * `0` means that the element should be focusable and reachable via sequential keyboard navigation, but its relative order is defined by the platform convention;
        /// * a _positive value_ means that the element should be focusable and reachable via sequential keyboard navigation; the order in which the elements are focused is the increasing value of the tabindex. If several elements share the same tabindex, their relative order follows their relative positions in the document.
        tabindex: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// Contains a text representing advisory information related to the element it belongs to. Such information can typically, but not necessarily, be presented to the user as a tooltip.
        title: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// An enumerated attribute that is used to specify whether an element's attribute values and the values of its Text node children are to be translated when the page is localized, or whether to leave them unchanged. It can have the following values:
        /// * empty string or `yes`, which indicates that the element will be translated.
        /// * `no`, which indicates that the element will not be translated.
        translate: impl Into<MaybeDyn<Cow<'static, str>>>,
        /// An enumerated attribute used to control the on-screen virtual keyboard behavior on devices such as tablets, mobile phones, or other devices where a hardware keyboard may not be available for elements that its content is editable (for example, it is an `<input>` or `<textarea>` element, or an element with the `contenteditable` attribute set).
        /// `auto` or an _empty string_, which automatically shows the virtual keyboard when the element is focused or tapped.
        /// `manual`, which decouples focus and tap on the element from the virtual keyboard's state.
        virtualkeyboardpolicy: impl Into<MaybeDyn<Cow<'static, str>>>,
    }
}

/// A trait that is implemented for all SVG elements and which provides all the global SVG
/// attributes.
///
/// Reference: <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute>
pub trait SvgGlobalAttributes: SetAttribute + Sized {
    impl_attributes! {
        accentHeight("accent-height"): impl Into<MaybeDyn<Cow<'static, str>>>,
        accumulate: impl Into<MaybeDyn<Cow<'static, str>>>,
        additive: impl Into<MaybeDyn<Cow<'static, str>>>,
        alignmentBaseline("alignment-baseline"): impl Into<MaybeDyn<Cow<'static, str>>>,
        alphabetic: impl Into<MaybeDyn<Cow<'static, str>>>,
        amplitude: impl Into<MaybeDyn<Cow<'static, str>>>,
        arabicForm("arabic-form"): impl Into<MaybeDyn<Cow<'static, str>>>,
        ascent: impl Into<MaybeDyn<Cow<'static, str>>>,
        attributeName("attributeName"): impl Into<MaybeDyn<Cow<'static, str>>>,
        attributeType("attributeType"): impl Into<MaybeDyn<Cow<'static, str>>>,
        azimuth: impl Into<MaybeDyn<Cow<'static, str>>>,
        baseFrequency("baseFrequency"): impl Into<MaybeDyn<Cow<'static, str>>>,
        baselineShift("baseline-shift"): impl Into<MaybeDyn<Cow<'static, str>>>,
        baseProfile("baseProfile"): impl Into<MaybeDyn<Cow<'static, str>>>,
        bbox: impl Into<MaybeDyn<Cow<'static, str>>>,
        begin: impl Into<MaybeDyn<Cow<'static, str>>>,
        bias: impl Into<MaybeDyn<Cow<'static, str>>>,
        by: impl Into<MaybeDyn<Cow<'static, str>>>,
        calcMode("calcMode"): impl Into<MaybeDyn<Cow<'static, str>>>,
        capHeight("cap-height"): impl Into<MaybeDyn<Cow<'static, str>>>,
        class: impl Into<MaybeDyn<Cow<'static, str>>>,
        clipPathUnits("clipPathUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        clipPath("clip-path"): impl Into<MaybeDyn<Cow<'static, str>>>,
        clipRule("clip-rule"): impl Into<MaybeDyn<Cow<'static, str>>>,
        color: impl Into<MaybeDyn<Cow<'static, str>>>,
        colorInterpolation("color-interpolation"): impl Into<MaybeDyn<Cow<'static, str>>>,
        colorInterpolationFilters("color-interpolation-filters"): impl Into<MaybeDyn<Cow<'static, str>>>,
        colorProfile("color-profile"): impl Into<MaybeDyn<Cow<'static, str>>>,
        colorRendering("color-rendering"): impl Into<MaybeDyn<Cow<'static, str>>>,
        crossorigin: impl Into<MaybeDyn<Cow<'static, str>>>,
        cursor: impl Into<MaybeDyn<Cow<'static, str>>>,
        cx: impl Into<MaybeDyn<Cow<'static, str>>>,
        cy: impl Into<MaybeDyn<Cow<'static, str>>>,
        d: impl Into<MaybeDyn<Cow<'static, str>>>,
        decelerate: impl Into<MaybeDyn<Cow<'static, str>>>,
        descent: impl Into<MaybeDyn<Cow<'static, str>>>,
        diffuseConstant("diffuseConstant"): impl Into<MaybeDyn<Cow<'static, str>>>,
        direction: impl Into<MaybeDyn<Cow<'static, str>>>,
        display: impl Into<MaybeDyn<Cow<'static, str>>>,
        divisor: impl Into<MaybeDyn<Cow<'static, str>>>,
        dominantBaseline("dominant-baseline"): impl Into<MaybeDyn<Cow<'static, str>>>,
        dur: impl Into<MaybeDyn<Cow<'static, str>>>,
        dx: impl Into<MaybeDyn<Cow<'static, str>>>,
        dy: impl Into<MaybeDyn<Cow<'static, str>>>,
        edgeMode("edgeMode"): impl Into<MaybeDyn<Cow<'static, str>>>,
        elevation: impl Into<MaybeDyn<Cow<'static, str>>>,
        enableBackground("enable-background"): impl Into<MaybeDyn<Cow<'static, str>>>,
        end: impl Into<MaybeDyn<Cow<'static, str>>>,
        exponent: impl Into<MaybeDyn<Cow<'static, str>>>,
        fill: impl Into<MaybeDyn<Cow<'static, str>>>,
        fillOpacity("fill-opacity"): impl Into<MaybeDyn<Cow<'static, str>>>,
        fillRule("fill-rule"): impl Into<MaybeDyn<Cow<'static, str>>>,
        filter: impl Into<MaybeDyn<Cow<'static, str>>>,
        filterUnits("filterUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        floodColor("flood-color"): impl Into<MaybeDyn<Cow<'static, str>>>,
        floodOpacity("flood-opacity"): impl Into<MaybeDyn<Cow<'static, str>>>,
        fontFamily("font-family"): impl Into<MaybeDyn<Cow<'static, str>>>,
        fontSize("font-size"): impl Into<MaybeDyn<Cow<'static, str>>>,
        fontSizeAdjust("font-size-adjust"): impl Into<MaybeDyn<Cow<'static, str>>>,
        fontStretch("font-stretch"): impl Into<MaybeDyn<Cow<'static, str>>>,
        fontStyle("font-style"): impl Into<MaybeDyn<Cow<'static, str>>>,
        fontVariant("font-variant"): impl Into<MaybeDyn<Cow<'static, str>>>,
        fontWeight("font-weight"): impl Into<MaybeDyn<Cow<'static, str>>>,
        format: impl Into<MaybeDyn<Cow<'static, str>>>,
        from: impl Into<MaybeDyn<Cow<'static, str>>>,
        fr: impl Into<MaybeDyn<Cow<'static, str>>>,
        fx: impl Into<MaybeDyn<Cow<'static, str>>>,
        fy: impl Into<MaybeDyn<Cow<'static, str>>>,
        g1: impl Into<MaybeDyn<Cow<'static, str>>>,
        g2: impl Into<MaybeDyn<Cow<'static, str>>>,
        glyphName("glyph-name"): impl Into<MaybeDyn<Cow<'static, str>>>,
        glyphOrientationHorizontal("glyph-orientation-horizontal"): impl Into<MaybeDyn<Cow<'static, str>>>,
        glyphOrientationVertical("glyph-orientation-vertical"): impl Into<MaybeDyn<Cow<'static, str>>>,
        glyphRef: impl Into<MaybeDyn<Cow<'static, str>>>,
        gradientTransform("gradientTransform"): impl Into<MaybeDyn<Cow<'static, str>>>,
        gradientUnits("gradientUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        hanging: impl Into<MaybeDyn<Cow<'static, str>>>,
        height: impl Into<MaybeDyn<Cow<'static, str>>>,
        href: impl Into<MaybeDyn<Cow<'static, str>>>,
        hreflang: impl Into<MaybeDyn<Cow<'static, str>>>,
        horizAdvX("horiz-adv-x"): impl Into<MaybeDyn<Cow<'static, str>>>,
        horizOriginX("horiz-origin-x"): impl Into<MaybeDyn<Cow<'static, str>>>,
        id: impl Into<MaybeDyn<Cow<'static, str>>>,
        ideographic: impl Into<MaybeDyn<Cow<'static, str>>>,
        imageRendering("image-rendering"): impl Into<MaybeDyn<Cow<'static, str>>>,
        in_: impl Into<MaybeDyn<Cow<'static, str>>>,
        in2: impl Into<MaybeDyn<Cow<'static, str>>>,
        intercept: impl Into<MaybeDyn<Cow<'static, str>>>,
        k: impl Into<MaybeDyn<Cow<'static, str>>>,
        k1: impl Into<MaybeDyn<Cow<'static, str>>>,
        k2: impl Into<MaybeDyn<Cow<'static, str>>>,
        k3: impl Into<MaybeDyn<Cow<'static, str>>>,
        k4: impl Into<MaybeDyn<Cow<'static, str>>>,
        kernelMatrix("kernelMatrix"): impl Into<MaybeDyn<Cow<'static, str>>>,
        kernelUnitLength("kernelUnitLength"): impl Into<MaybeDyn<Cow<'static, str>>>,
        kerning: impl Into<MaybeDyn<Cow<'static, str>>>,
        keyPoints("keyPoints"): impl Into<MaybeDyn<Cow<'static, str>>>,
        keySplines("keySplines"): impl Into<MaybeDyn<Cow<'static, str>>>,
        keyTimes("keyTimes"): impl Into<MaybeDyn<Cow<'static, str>>>,
        lang: impl Into<MaybeDyn<Cow<'static, str>>>,
        lengthAdjust("lengthAdjust"): impl Into<MaybeDyn<Cow<'static, str>>>,
        letterSpacing("letter-spacing"): impl Into<MaybeDyn<Cow<'static, str>>>,
        lightingColor("lighting-color"): impl Into<MaybeDyn<Cow<'static, str>>>,
        limitingConeAngle("limitingConeAngle"): impl Into<MaybeDyn<Cow<'static, str>>>,
        local: impl Into<MaybeDyn<Cow<'static, str>>>,
        markerEnd("marker-end"): impl Into<MaybeDyn<Cow<'static, str>>>,
        markerMid("marker-mid"): impl Into<MaybeDyn<Cow<'static, str>>>,
        markerStart("marker-start"): impl Into<MaybeDyn<Cow<'static, str>>>,
        markerHeight("markerHeight"): impl Into<MaybeDyn<Cow<'static, str>>>,
        markerUnits("markerUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        markerWidth("markerWidth"): impl Into<MaybeDyn<Cow<'static, str>>>,
        mask: impl Into<MaybeDyn<Cow<'static, str>>>,
        maskContentUnits("maskContentUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        maskUnits("maskUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        mathematical: impl Into<MaybeDyn<Cow<'static, str>>>,
        max: impl Into<MaybeDyn<Cow<'static, str>>>,
        media: impl Into<MaybeDyn<Cow<'static, str>>>,
        method: impl Into<MaybeDyn<Cow<'static, str>>>,
        min: impl Into<MaybeDyn<Cow<'static, str>>>,
        mode: impl Into<MaybeDyn<Cow<'static, str>>>,
        name: impl Into<MaybeDyn<Cow<'static, str>>>,
        numOctaves("numOctaves"): impl Into<MaybeDyn<Cow<'static, str>>>,
        offset: impl Into<MaybeDyn<Cow<'static, str>>>,
        opacity: impl Into<MaybeDyn<Cow<'static, str>>>,
        operator: impl Into<MaybeDyn<Cow<'static, str>>>,
        order: impl Into<MaybeDyn<Cow<'static, str>>>,
        orient: impl Into<MaybeDyn<Cow<'static, str>>>,
        orientation: impl Into<MaybeDyn<Cow<'static, str>>>,
        origin: impl Into<MaybeDyn<Cow<'static, str>>>,
        overflow: impl Into<MaybeDyn<Cow<'static, str>>>,
        overlinePosition("overline-position"): impl Into<MaybeDyn<Cow<'static, str>>>,
        overlineThickness("overline-thickness"): impl Into<MaybeDyn<Cow<'static, str>>>,
        panose1("panose-1"): impl Into<MaybeDyn<Cow<'static, str>>>,
        paintOrder("paint-order"): impl Into<MaybeDyn<Cow<'static, str>>>,
        path: impl Into<MaybeDyn<Cow<'static, str>>>,
        pathLength("pathLength"): impl Into<MaybeDyn<Cow<'static, str>>>,
        patternContentUnits("patternContentUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        patternTransform("patternTransform"): impl Into<MaybeDyn<Cow<'static, str>>>,
        patternUnits("patternUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        ping: impl Into<MaybeDyn<Cow<'static, str>>>,
        pointerEvents("pointer-events"): impl Into<MaybeDyn<Cow<'static, str>>>,
        points: impl Into<MaybeDyn<Cow<'static, str>>>,
        pointsAtX("pointsAtX"): impl Into<MaybeDyn<Cow<'static, str>>>,
        pointsAtY("pointsAtY"): impl Into<MaybeDyn<Cow<'static, str>>>,
        pointsAtZ("pointsAtZ"): impl Into<MaybeDyn<Cow<'static, str>>>,
        preserveAlpha("preserveAlpha"): impl Into<MaybeDyn<Cow<'static, str>>>,
        preserveAspectRatio("preserveAspectRatio"): impl Into<MaybeDyn<Cow<'static, str>>>,
        primitiveUnits("primitiveUnits"): impl Into<MaybeDyn<Cow<'static, str>>>,
        r: impl Into<MaybeDyn<Cow<'static, str>>>,
        radius: impl Into<MaybeDyn<Cow<'static, str>>>,
        referrerPolicy("referrerPolicy"): impl Into<MaybeDyn<Cow<'static, str>>>,
        refX("refX"): impl Into<MaybeDyn<Cow<'static, str>>>,
        refY("refY"): impl Into<MaybeDyn<Cow<'static, str>>>,
        rel: impl Into<MaybeDyn<Cow<'static, str>>>,
        renderingIntent("rendering-intent"): impl Into<MaybeDyn<Cow<'static, str>>>,
        repeatCount("repeatCount"): impl Into<MaybeDyn<Cow<'static, str>>>,
        repeatDur("repeatDur"): impl Into<MaybeDyn<Cow<'static, str>>>,
        requiredExtensions("requiredExtensions"): impl Into<MaybeDyn<Cow<'static, str>>>,
        requiredFeatures("requiredFeatures"): impl Into<MaybeDyn<Cow<'static, str>>>,
        restart: impl Into<MaybeDyn<Cow<'static, str>>>,
        result: impl Into<MaybeDyn<Cow<'static, str>>>,
        rotate: impl Into<MaybeDyn<Cow<'static, str>>>,
        rx: impl Into<MaybeDyn<Cow<'static, str>>>,
        ry: impl Into<MaybeDyn<Cow<'static, str>>>,
        scale: impl Into<MaybeDyn<Cow<'static, str>>>,
        seed: impl Into<MaybeDyn<Cow<'static, str>>>,
        shapeRendering("shape-rendering"): impl Into<MaybeDyn<Cow<'static, str>>>,
        slope: impl Into<MaybeDyn<Cow<'static, str>>>,
        spacing: impl Into<MaybeDyn<Cow<'static, str>>>,
        specularConstant("specularConstant"): impl Into<MaybeDyn<Cow<'static, str>>>,
        specularExponent("specularExponent"): impl Into<MaybeDyn<Cow<'static, str>>>,
        speed: impl Into<MaybeDyn<Cow<'static, str>>>,
        spreadMethod("spreadMethod"): impl Into<MaybeDyn<Cow<'static, str>>>,
        startOffset("startOffset"): impl Into<MaybeDyn<Cow<'static, str>>>,
        stdDeviation("stdDeviation"): impl Into<MaybeDyn<Cow<'static, str>>>,
        stemh: impl Into<MaybeDyn<Cow<'static, str>>>,
        stemv: impl Into<MaybeDyn<Cow<'static, str>>>,
        stitchTiles("stitchTiles"): impl Into<MaybeDyn<Cow<'static, str>>>,
        stopColor("stop-color"): impl Into<MaybeDyn<Cow<'static, str>>>,
        stopOpacity("stop-opacity"): impl Into<MaybeDyn<Cow<'static, str>>>,
        strikethroughPosition("strikethrough-position"): impl Into<MaybeDyn<Cow<'static, str>>>,
        strikethroughThickness("strikethrough-thickness"): impl Into<MaybeDyn<Cow<'static, str>>>,
        string: impl Into<MaybeDyn<Cow<'static, str>>>,
        stroke: impl Into<MaybeDyn<Cow<'static, str>>>,
        strokeDasharray("stroke-dasharray"): impl Into<MaybeDyn<Cow<'static, str>>>,
        strokeDashoffset("stroke-dashoffset"): impl Into<MaybeDyn<Cow<'static, str>>>,
        strokeLinecap("stroke-linecap"): impl Into<MaybeDyn<Cow<'static, str>>>,
        strokeLinejoin("stroke-linejoin"): impl Into<MaybeDyn<Cow<'static, str>>>,
        strokeMiterlimit("stroke-miterlimit"): impl Into<MaybeDyn<Cow<'static, str>>>,
        strokeOpacity("stroke-opacity"): impl Into<MaybeDyn<Cow<'static, str>>>,
        strokeWidth("stroke-width"): impl Into<MaybeDyn<Cow<'static, str>>>,
        style: impl Into<MaybeDyn<Cow<'static, str>>>,
        surfaceScale("surfaceScale"): impl Into<MaybeDyn<Cow<'static, str>>>,
        systemLanguage("systemLanguage"): impl Into<MaybeDyn<Cow<'static, str>>>,
        tabindex: impl Into<MaybeDyn<Cow<'static, str>>>,
        tableValues("tableValues"): impl Into<MaybeDyn<Cow<'static, str>>>,
        target: impl Into<MaybeDyn<Cow<'static, str>>>,
        targetX("targetX"): impl Into<MaybeDyn<Cow<'static, str>>>,
        targetY("targetY"): impl Into<MaybeDyn<Cow<'static, str>>>,
        textAnchor("text-anchor"): impl Into<MaybeDyn<Cow<'static, str>>>,
        textDecoration("text-decoration"): impl Into<MaybeDyn<Cow<'static, str>>>,
        textRendering("text-rendering"): impl Into<MaybeDyn<Cow<'static, str>>>,
        textLength("textLength"): impl Into<MaybeDyn<Cow<'static, str>>>,
        to: impl Into<MaybeDyn<Cow<'static, str>>>,
        transform: impl Into<MaybeDyn<Cow<'static, str>>>,
        transformOrigin("transform-origin"): impl Into<MaybeDyn<Cow<'static, str>>>,
        type_: impl Into<MaybeDyn<Cow<'static, str>>>,
        u1: impl Into<MaybeDyn<Cow<'static, str>>>,
        u2: impl Into<MaybeDyn<Cow<'static, str>>>,
        underlinePosition("underline-position"): impl Into<MaybeDyn<Cow<'static, str>>>,
        underlineThickness("underline-thickness"): impl Into<MaybeDyn<Cow<'static, str>>>,
        unicode: impl Into<MaybeDyn<Cow<'static, str>>>,
        unicodeBidi("unicode-bidi"): impl Into<MaybeDyn<Cow<'static, str>>>,
        unicodeRange("unicode-range"): impl Into<MaybeDyn<Cow<'static, str>>>,
        unitsPerEm("units-per-em"): impl Into<MaybeDyn<Cow<'static, str>>>,
        vAlphabetic("v-alphabetic"): impl Into<MaybeDyn<Cow<'static, str>>>,
        vHanging("v-hanging"): impl Into<MaybeDyn<Cow<'static, str>>>,
        vIdeographic("v-ideographic"): impl Into<MaybeDyn<Cow<'static, str>>>,
        vMathematical("v-mathematical"): impl Into<MaybeDyn<Cow<'static, str>>>,
        values: impl Into<MaybeDyn<Cow<'static, str>>>,
        vectorEffect("vector-effect"): impl Into<MaybeDyn<Cow<'static, str>>>,
        version: impl Into<MaybeDyn<Cow<'static, str>>>,
        vertAdvY("vert-adv-y"): impl Into<MaybeDyn<Cow<'static, str>>>,
        vertOriginX("vert-origin-x"): impl Into<MaybeDyn<Cow<'static, str>>>,
        vertOriginY("vert-origin-y"): impl Into<MaybeDyn<Cow<'static, str>>>,
        viewBox: impl Into<MaybeDyn<Cow<'static, str>>>,
        visibility: impl Into<MaybeDyn<Cow<'static, str>>>,
        width: impl Into<MaybeDyn<Cow<'static, str>>>,
        widths: impl Into<MaybeDyn<Cow<'static, str>>>,
        wordSpacing("word-spacing"): impl Into<MaybeDyn<Cow<'static, str>>>,
        writingMode("writing-mode"): impl Into<MaybeDyn<Cow<'static, str>>>,
        x: impl Into<MaybeDyn<Cow<'static, str>>>,
        xHeight("x-height"): impl Into<MaybeDyn<Cow<'static, str>>>,
        x1: impl Into<MaybeDyn<Cow<'static, str>>>,
        x2: impl Into<MaybeDyn<Cow<'static, str>>>,
        xChannelSelector("xChannelSelector"): impl Into<MaybeDyn<Cow<'static, str>>>,
        xmlBase("xml:base"): impl Into<MaybeDyn<Cow<'static, str>>>,
        xmlLang("xml:lang"): impl Into<MaybeDyn<Cow<'static, str>>>,
        xmlSpace("xml:space"): impl Into<MaybeDyn<Cow<'static, str>>>,
        y: impl Into<MaybeDyn<Cow<'static, str>>>,
        y1: impl Into<MaybeDyn<Cow<'static, str>>>,
        y2: impl Into<MaybeDyn<Cow<'static, str>>>,
        yChannelSelector("yChannelSelector"): impl Into<MaybeDyn<Cow<'static, str>>>,
        zoomAndPan("zoomAndPan"): impl Into<MaybeDyn<Cow<'static, str>>>,
    }
}

/// Attributes that are available on all elements.
pub trait GlobalAttributes: SetAttribute + Sized {
    /// Set attribute `name` with `value`.
    fn attr(mut self, name: &'static str, value: impl Into<MaybeDyn<Cow<'static, str>>>) -> Self {
        self.set_attribute(name, value.into());
        self
    }

    /// Set attribute `name` with `value` from Option.
    fn attr_opt(
        mut self,
        name: &'static str,
        value: impl Into<MaybeDyn<Option<Cow<'static, str>>>>,
    ) -> Self {
        self.set_attribute(name, value.into());
        self
    }

    /// Set attribute `name` with `value`.
    fn bool_attr(mut self, name: &'static str, value: impl Into<MaybeDyn<bool>>) -> Self {
        self.set_attribute(name, value.into());
        self
    }

    /// Set JS property `name` with `value`.
    fn prop(mut self, name: &'static str, value: impl Into<MaybeDyn<JsValue>>) -> Self {
        self.set_attribute(name, value.into());
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

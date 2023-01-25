//! Definitions for HTML attributes that can be used with the [`attr`] directive or with nothing at
//! all (which uses [`attr`] by default).

#![allow(non_snake_case)]

use std::borrow::Cow;

use sycamore_core::generic_node::GenericNode;
use sycamore_core::noderef::NodeRef;

use super::elements::{HtmlElement, SvgElement};
use super::{Attributes, SetAttribute, WebElement};
use crate::web_node::WebNode;
use crate::ElementBuilder;

/// Codegen the methods for the attributes.
macro_rules! define_attributes {
    (
        $(
            $(#[$attr:meta])*
            $id:ident $(($name:literal))? : $ty:ident,
        )*
    ) => {
        $(
            define_attributes! {
                $(#[$attr])*
                fn $id $(($name))? : $ty
            }
        )*
    };
    (
        $(#[$attr:meta])*
        fn $id:ident : String
    ) => {
        $(#[$attr])*
        fn $id(self, v: impl Into<Cow<'static, str>>) -> Self {
            self.set_attribute(stringify!($id).into(), v.into());
            self
        }
    };
    (
        $(#[$attr:meta])*
        fn $id:ident : bool
    ) => {
        $(#[$attr])*
        fn $id(self, v: bool) -> Self {
            if v {
                self.set_attribute(stringify!($id).into(), Cow::Borrowed(""));
            } else {
                self.remove_attribute(stringify!($id).into());
            }
            self
        }
    };
    (
        $(#[$attr:meta])*
        fn $id:ident ($name:literal) : String
    ) => {
        $(#[$attr])*
        fn $id(self, v: impl Into<Cow<'static, str>>) -> Self {
            self.set_attribute(Cow::Borrowed($name), v.into());
            self
        }
    };
    (
        $(#[$attr:meta])*
        fn $id:ident ($name:literal) : bool
    ) => {
        $(#[$attr])*
        fn $id(self, v: bool) -> Self {
            if v {
                self.set_attribute(Cow::Borrowed($name), Cow::Borrowed(""));
            } else {
                self.remove_attribute(Cow::Borrowed($name));
            }
            self
        }
    };
}

/// The global attribute for both HTML and SVG.
pub trait GlobalAttributes: SetAttribute + Sized {
    fn custom_attr(self, name: &'static str, value: impl Into<Cow<'static, str>>) -> Self;

    /// Set the inner HTML of the element.
    ///
    /// TODO (docs): Warn about potential XSS vulnerabilities.
    fn dangerously_set_inner_html(self, html: impl Into<Cow<'static, str>>) -> Self;

    /// Set a [`NodeRef`] to this element.
    fn _ref(self, v: &NodeRef<WebNode>) -> Self;

    // Some attributes are shared for both HTML and SVG elements.
    // We declare them here to prevent declaring them twice for both HTML and SVG.
    //
    // This way, we don't have any ambiguities when calling the attributes.
    define_attributes! {
        /// The `class` global attribute is a space-separated list of the case-sensitive classes of the element.
        /// Classes allow CSS and JavaScript to select and access specific elements via the class selectors or functions
        /// like the DOM method `document.getElementsByClassName`.
        class: String,
        /// The `id` global attribute defines an identifier (ID) which must be unique in the whole document.
        /// Its purpose is to identify the element when linking (using a fragment identifier), scripting, or styling (with CSS).
        id: String,
        /// The `style` global attribute contains CSS styling declarations to be applied to the element. Note that it is recommended for styles to be defined in a separate file or files.
        /// This attribute and the `<style>` element have mainly the purpose of allowing for quick styling, for example for testing purposes.
        style: String,
        tabindex: String,
        _type("type"): String,
    }
}
impl<'a, T> GlobalAttributes for ElementBuilder<'a, T>
where
    T: WebElement,
{
    fn custom_attr(self, name: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        self.set_attribute(name.into(), value.into());
        self
    }
    fn dangerously_set_inner_html(self, html: impl Into<Cow<'static, str>>) -> Self {
        self.as_node().dangerously_set_inner_html(html.into());
        self
    }
    fn _ref(self, v: &NodeRef<WebNode>) -> Self {
        v.set(self.as_node().clone());
        self
    }
}
impl<'a, T> GlobalAttributes for Attributes<'a, T>
where
    T: WebElement,
{
    fn custom_attr(self, name: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        let value = value.into();
        self.add_fn(move |builder| {
            builder.custom_attr(name, value);
        });
        self
    }
    fn dangerously_set_inner_html(self, html: impl Into<Cow<'static, str>>) -> Self {
        let html = html.into();
        self.add_fn(move |builder| {
            builder.dangerously_set_inner_html(html);
        });
        self
    }
    fn _ref(self, v: &NodeRef<WebNode>) -> Self {
        let v = v.clone();
        self.add_fn(move |builder| {
            builder._ref(&v);
        });
        self
    }
}

/// The global HTML attributes.
pub trait HtmlGlobalAttributes: SetAttribute + Sized {
    define_attributes! {
        accesskey: String,
        autocapitalize: String,
        autofocus: bool,
        /// The `contenteditable` global attribute is an enumerated attribute indicating if the element should be editable by the user.
        /// If so, the browser modifies its widget to allow editing.
        ///
        /// The attribute must take one of the following values:
        /// * `true` or an _empty string_, which indicates that the element is editable.
        /// * `false`, which indicates that the element is not editable.
        ///
        /// If this attribute is missing or its value is invalid, its value is inherited from its parent element: so the element is editable if its parent is editable.
        ///
        /// Note that although its allowed values include `true` and `false`, this attribute is an enumerated one and not a Boolean one.
        contenteditable: String,
        contextmenu: String,
        dir: String,
        draggable: String,
        enterkeyhint: String,
        exportparts: String,
        /// The `hidden` global attribute is an enumerated attribute indicating that the browser should not render the contents of the element.
        /// For example, it can be used to hide elements of the page that can't be used until the login process has been completed.
        hidden: bool,
        href: String,
        inert: bool,
        inputmode: String,
        is: String,
        itemid: String,
        itemprop: String,
        itemref: String,
        itemscope: bool,
        itemtype: String,
        lang: String,
        nonce: String,
        part: String,
        role: String,
        slot: String,
        spellcheck: String,
        title: String,
        translate: String,
        virtualkeyboardpolicy: String,
    }

    /// Insert an `aria-*` attribute.
    fn aria(self, name: &'static str, v: impl Into<Cow<'static, str>>) -> Self {
        let name = String::from("aria-") + name;
        self.set_attribute(Cow::Owned(name), v.into());
        self
    }
    /// Insert a `data-*` attribute.
    fn data(self, name: &'static str, v: impl Into<Cow<'static, str>>) -> Self {
        let name = String::from("data-") + name;
        self.set_attribute(Cow::Owned(name), v.into());
        self
    }
    /// Insert a custom attribute.
    fn custom(self, name: &'static str, v: impl Into<Cow<'static, str>>) -> Self {
        self.set_attribute(Cow::Borrowed(name), v.into());
        self
    }
}

impl<'a, T> HtmlGlobalAttributes for ElementBuilder<'a, T> where T: HtmlElement {}
impl<'a, T> HtmlGlobalAttributes for Attributes<'a, T> where T: HtmlElement {}

pub trait SvgGlobalAttributes: SetAttribute + Sized {
    define_attributes! {
        accent_height: String,
        accumulate: String,
        additive: String,
        alignment_baseline: String,
        alphabetic: String,
        amplitude: String,
        arabic_form: String,
        ascent: String,
        attributeName: String,
        attributeType: String,
        azimuth: String,
        baseFrequency: String,
        baseline_shift: String,
        baseProfile: String,
        bbox: String,
        begin: String,
        bias: String,
        by: String,
        calcMode: String,
        cap_height: String,
        clip: String,
        clipPathUnits: String,
        clip_path: String,
        clip_rule: String,
        color: String,
        color_interpolation: String,
        color_interpolation_filters: String,
        color_profile: String,
        color_rendering: String,
        contentScriptType: String,
        contentStyleType: String,
        crossorigin: String,
        cursor: String,
        cx: String,
        cy: String,
        d: String,
        decelerate: String,
        descent: String,
        diffuseConstant: String,
        direction: String,
        display: String,
        divisor: String,
        dominant_baseline: String,
        dur: String,
        dx: String,
        dy: String,
        edgeMode: String,
        elevation: String,
        enable_background: String,
        end: String,
        exponent: String,
        fill: String,
        fill_opacity: String,
        fill_rule: String,
        filter: String,
        filterRes: String,
        filterUnits: String,
        flood_color: String,
        flood_opacity: String,
        font_family: String,
        font_size: String,
        font_size_adjust: String,
        font_stretch: String,
        font_style: String,
        font_variant: String,
        font_weight: String,
        format: String,
        from: String,
        fr: String,
        fx: String,
        fy: String,
        g1: String,
        g2: String,
        glyph_name: String,
        glyph_orientation_horizontal: String,
        glyph_orientation_vertical: String,
        glyphRef: String,
        gradientTransform: String,
        gradientUnits: String,
        hanging: String,
        height: String,
        href: String,
        hreflang: String,
        horiz_adv_x: String,
        horiz_origin_x: String,
        ideographic: String,
        image_rendering: String,
        _in("in"): String,
        in2: String,
        intercept: String,
        k: String,
        k1: String,
        k2: String,
        k3: String,
        k4: String,
        kernelMatrix: String,
        kernelUnitLength: String,
        kerning: String,
        keyPoints: String,
        keySplines: String,
        keyTimes: String,
        lang: String,
        lengthAdjust: String,
        letter_spacing: String,
        lighting_color: String,
        limitingConeAngle: String,
        local: String,
        marker_end: String,
        marker_mid: String,
        marker_start: String,
        markerHeight: String,
        markerUnits: String,
        markerWidth: String,
        mask: String,
        maskContentUnits: String,
        maskUnits: String,
        mathematical: String,
        max: String,
        media: String,
        method: String,
        min: String,
        mode: String,
        name: String,
        numOctaves: String,
        offset: String,
        opacity: String,
        operator: String,
        order: String,
        orient: String,
        orientation: String,
        origin: String,
        overflow: String,
        overline_position: String,
        overline_thickness: String,
        panose_1: String,
        paint_order: String,
        path: String,
        pathLength: String,
        patternContentUnits: String,
        patternTransform: String,
        patternUnits: String,
        ping: String,
        pointer_events: String,
        points: String,
        pointsAtX: String,
        pointsAtY: String,
        pointsAtZ: String,
        preserveAlpha: String,
        preserveAspectRatio: String,
        primitiveUnits: String,
        r: String,
        radius: String,
        referrerPolicy: String,
        refX: String,
        refY: String,
        rel: String,
        rendering_intent: String,
        repeatCount: String,
        repeatDur: String,
        requiredExtensions: String,
        requiredFeatures: String,
        restart: String,
        result: String,
        rotate: String,
        rx: String,
        ry: String,
        scale: String,
        seed: String,
        shape_rendering: String,
        slope: String,
        spacing: String,
        specularConstant: String,
        specularExponent: String,
        speed: String,
        spreadMethod: String,
        startOffset: String,
        stdDeviation: String,
        stemh: String,
        stemv: String,
        stitchTiles: String,
        stop_color: String,
        stop_opacity: String,
        strikethrough_position: String,
        strikethrough_thickness: String,
        string: String,
        stroke: String,
        stroke_dasharray: String,
        stroke_dashoffset: String,
        stroke_linecap: String,
        stroke_linejoin: String,
        stroke_miterlimit: String,
        stroke_opacity: String,
        stroke_width: String,
        surfaceScale: String,
        systemLanguage: String,
        tableValues: String,
        target: String,
        targetX: String,
        targetY: String,
        text_anchor: String,
        text_decoration: String,
        text_rendering: String,
        textLength: String,
        to: String,
        transform: String,
        transform_origin: String,
        u1: String,
        u2: String,
        underline_position: String,
        underline_thickness: String,
        unicode: String,
        unicode_bidi: String,
        unicode_range: String,
        units_per_em: String,
        v_alphabetic: String,
        v_hanging: String,
        v_ideographic: String,
        v_mathematical: String,
        values: String,
        vector_effect: String,
        version: String,
        vert_adv_y: String,
        vert_origin_x: String,
        vert_origin_y: String,
        viewBox: String,
        viewTarget: String,
        visibility: String,
        width: String,
        widths: String,
        word_spacing: String,
        writing_mode: String,
        x: String,
        x_height: String,
        x1: String,
        x2: String,
        xChannelSelector: String,
        y: String,
        y1: String,
        y2: String,
        yChannelSelector: String,
        z: String,
        zoomAndPan: String,
    }
}

impl<'a, T> SvgGlobalAttributes for ElementBuilder<'a, T> where T: SvgElement {}
impl<'a, T> SvgGlobalAttributes for Attributes<'a, T> where T: SvgElement {}

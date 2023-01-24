//! Definitions for HTML attributes that can be used with the [`attr`] directive or with nothing at
//! all (which uses [`attr`] by default).

#![allow(non_snake_case)]

use std::borrow::Cow;

use sycamore_core2::elements::AsNode;
use sycamore_core2::generic_node::GenericNode;

use super::elements::{HtmlElement, SvgElement};
use crate::ElementBuilder;
use crate::web_node::WebNode;

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
            self.as_node().set_attribute(stringify!($id).into(), v.into());
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
                self.as_node().set_attribute(stringify!($id).into(), Cow::Borrowed(""));
            } else {
                self.as_node().remove_attribute(stringify!($id).into());
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
            self.as_node().set_attribute(Cow::Borrowed($name), v.into());
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
                self.as_node().set_attribute(Cow::Borrowed($name), Cow::Borrowed(""));
            } else {
                self.as_node().remove_attribute(Cow::Borrowed($name));
            }
            self
        }
    };
}

/// The global HTML attributes.
pub trait GlobalAttributes: AsNode<WebNode> + Sized {
    define_attributes! {
        accesskey: String,
        autocapitalize: String,
        autofocus: bool,
        class: String,
        contenteditable: String,
        contextmenu: String,
        dir: String,
        draggable: String,
        enterkeyhint: String,
        exportparts: String,
        hidden: bool,
        id: String,
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
        style: String,
        tabindex: String,
        title: String,
        translate: String,
        virtualkeyboardpolicy: String,
    }

    /// Insert an `aria-*` attribute.
    fn aria(self, name: &'static str, v: impl Into<Cow<'static, str>>) -> Self {
        let name = String::from("aria-") + name;
        self.as_node().set_attribute(Cow::Owned(name), v.into());
        self
    }
    /// Insert a `data-*` attribute.
    fn data(self, name: &'static str, v: impl Into<Cow<'static, str>>) -> Self {
        let name = String::from("data-") + name;
        self.as_node().set_attribute(Cow::Owned(name), v.into());
        self
    }
    /// Insert a custom attribute.
    fn custom(self, name: &'static str, v: impl Into<Cow<'static, str>>) -> Self {
        self.as_node().set_attribute(Cow::Borrowed(name), v.into());
        self
    }
}

impl<'a, T> GlobalAttributes for ElementBuilder<'a, T> where T: HtmlElement {}

pub trait GlobalSvgAttributes: AsNode<WebNode> + Sized {
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
        class: String,
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
        id: String,
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
        style: String,
        surfaceScale: String,
        systemLanguage: String,
        tabindex: String,
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
        _type("type"): String,
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

impl<'a, T> GlobalSvgAttributes for ElementBuilder<'a, T> where T: SvgElement {}

//! Definitions for HTML attributes that can be used with the [`attr`] directive or with nothing at
//! all (which uses [`attr`] by default).

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::borrow::Cow;

use sycamore_core2::attributes::{ApplyAttr, ApplyAttrDyn};
use sycamore_core2::elements::TypedElement;
use sycamore_core2::generic_node::GenericNode;
use sycamore_reactive::{create_effect, Scope};

use crate::web_node::WebNode;

/// The default attribute directive. This is the one that is used if no other attribute directive is
/// specified.
pub struct attr;

/// Represents an HTML attribute.
pub struct HtmlAttr<T> {
    name: Cow<'static, str>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> HtmlAttr<T> {
    pub const fn new(name: Cow<'static, str>) -> Self {
        Self {
            name,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T: ToString, E: TypedElement<WebNode>> ApplyAttr<'a, WebNode, T, E> for HtmlAttr<String> {
    fn apply(self, _cx: Scope<'a>, el: &WebNode, value: T) {
        el.set_attribute(self.name, value.to_string().into());
    }
}

impl<'a, T: ToString + 'a, E: TypedElement<WebNode>> ApplyAttrDyn<'a, WebNode, T, E>
    for HtmlAttr<String>
{
    fn apply_dyn(self, cx: Scope<'a>, el: &WebNode, mut value: Box<dyn FnMut() -> T + 'a>) {
        let el = el.clone();
        create_effect(cx, move || {
            el.set_attribute(self.name.clone(), value().to_string().into());
        });
    }
}

macro_rules! define_attributes {
    (
        $(
            $(#[$attr:meta])*
            $id:ident $(($name:literal))? : $ty:ty,
        )*
    ) => {
        impl attr {
            $(
                define_attributes! {
                    $(#[$attr])*
                    $id $(($name))? : $ty
                }
            )*
        }
    };
    (
        $(#[$attr:meta])*
        $id:ident : $ty:ty
    ) => {
        $(#[$attr])*
        pub const $id: HtmlAttr<$ty> = HtmlAttr::new(Cow::Borrowed(stringify!($id)));
    };
    (
        $(#[$attr:meta])*
        $id:ident($name:literal) : $ty:ty
    ) => {
        $(#[$attr])*
        pub const $id: HtmlAttr<$ty> = HtmlAttr::new(Cow::Borrowed($name));
    }
}

// Global attributes.
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
    tabindex: i32,
    title: String,
    translate: String,
    virtualkeyboardpolicy: String,
}

impl attr {
    pub fn aria(name: &'static str) -> HtmlAttr<String> {
        let name = String::from("aria-") + name;
        HtmlAttr::new(Cow::Owned(name))
    }
    pub fn data(name: &'static str) -> HtmlAttr<String> {
        let name = String::from("data-") + name;
        HtmlAttr::new(Cow::Owned(name))
    }
    pub fn custom(name: &'static str) -> HtmlAttr<String> {
        HtmlAttr::new(Cow::Borrowed(name))
    }
}

// SVG attributes.
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
    // class: String,
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
    // id: String,
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
    // lang: String,
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
    // style: String,
    surfaceScale: String,
    systemLanguage: String,
    // tabindex: String,
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

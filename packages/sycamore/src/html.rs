//! HTML tag definitions.
//!
//! _Documentation sources: <https://developer.mozilla.org/en-US/>_

pub use sycamore_web::on_mount;

use crate::builder::ElementBuilder;
use crate::generic_node::SycamoreElement;
use crate::prelude::*;
#[cfg(feature = "hydrate")]
use crate::utils::hydrate::{hydration_completed, with_no_hydration_context};
/// MBE for generating elements.
macro_rules! define_elements {
    (
        $ns:expr,
        $(
            $(#[$attr:meta])*
            $el:ident {
                $(
                    $(#[$attr_method:meta])*
                    $at:ident: $ty:path,
                )*
            },
        )*
    ) => {
        $(
            #[allow(non_camel_case_types)]
            #[doc = concat!("Build a [`<", stringify!($el), ">`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/", stringify!($el), ") element.")]
            $(#[$attr])*
            pub struct $el {}

            impl SycamoreElement for $el {
                const TAG_NAME: &'static str = stringify!($el);
                const NAME_SPACE: Option<&'static str> = $ns;
            }

            #[allow(non_snake_case)]
            #[doc = concat!("Create a [`<", stringify!($el), ">`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/", stringify!($el), ") element builder.")]
            $(#[$attr])*
            pub fn $el<'a, G: GenericNode>() -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G> {
                ElementBuilder::new(move |_| G::element::<$el>())
            }
        )*
    };
}

// A list of valid HTML5 elements (does not include removed or obsolete elements).
define_elements! {
    None,
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

// A list of valid SVG elements. Some elements are commented out because they conflict with the HTML
// elements.
define_elements! {
    Some("http://www.w3.org/2000/svg"),
    svg {},
    // a,
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
    // script {},
    set {},
    stop {},
    // style {},
    switch {},
    symbol {},
    text {},
    textPath {},
    // title {},
    tspan {},
    r#use {},
    view {},
}

/// Props for [`NoHydrate`].
#[cfg(feature = "hydrate")]
#[derive(Prop)]
pub struct NoHydrateProps<'a, G: GenericNode> {
    children: Children<'a, G>,
}

/// Render the children of this component in a scope that will not be hydrated.
///
/// When using `SsrNode`, this means that hydration markers won't be generated. When using
/// `HydrateNode`, this means that the entire sub-tree will be ignored. When using `DomNode`,
/// rendering proceeds as normal.
///
/// The children are wrapped inside a `<div>` element to prevent conflicts with surrounding
/// elements.
#[cfg(feature = "hydrate")]
#[component]
pub fn NoHydrate<'a, G: Html>(cx: Scope<'a>, props: NoHydrateProps<'a, G>) -> View<G> {
    use crate::utils::render;

    let node_ref = create_node_ref(cx);
    let v = view! { cx,
        // TODO: remove wrapper `div`. We currently cannot do that because otherwise
        // the node won't get inserted into the DOM.
        div(ref=node_ref) {}
    };
    if G::CLIENT_SIDE_HYDRATION && !hydration_completed() {
        // We don't want to hydrate the children, so we just do nothing.
    } else if G::USE_HYDRATION_CONTEXT {
        // If we have a hydration context, remove it in this scope so that hydration markers are not
        // generated.
        let nodes = with_no_hydration_context(|| props.children.call(cx));
        render::insert(cx, &node_ref.get_raw(), nodes, None, None, false);
    } else {
        // Just continue rendering as normal.
        let nodes = props.children.call(cx);
        render::insert(cx, &node_ref.get_raw(), nodes, None, None, false);
    };
    v
}

/// Props for [`NoSsr`].
#[cfg(feature = "hydrate")]
#[derive(Prop)]
pub struct NoSsrProps<'a, G: GenericNode> {
    children: Children<'a, G>,
}

/// Only render the children of this component in the browser.
/// The children are wrapped inside a `<div>` element to prevent conflicts with surrounding
/// elements.
#[cfg(feature = "hydrate")]
#[component]
pub fn NoSsr<'a, G: Html>(cx: Scope<'a>, props: NoSsrProps<'a, G>) -> View<G> {
    let node = if !G::IS_BROWSER {
        // We don't want to render the children, so we just do nothing.
        view! { cx, }
    } else if G::USE_HYDRATION_CONTEXT {
        // Since the nodes were not rendered on the server, there is nothing to hydrate.
        with_no_hydration_context(|| props.children.call(cx))
    } else {
        // Just continue rendering as normal.
        props.children.call(cx)
    };
    view! { cx,
        // TODO: remove wrapper `div`. We currently cannot do that because otherwise
        // the node won't get inserted into the DOM.
        div { (node) }
    }
}

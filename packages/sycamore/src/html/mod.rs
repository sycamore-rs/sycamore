//! HTML tag definitions.
//!
//! _Documentation sources: https://developer.mozilla.org/en-US/_

/// Represents an element.
pub trait SycamoreElement {
    const TAG_NAME: &'static str;
    const NAME_SPACE: Option<&'static str>;
}

/// MBE for generating elements.
macro_rules! define_elements {
    (
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
            pub struct $el;

            impl SycamoreElement for $el {
                const TAG_NAME: &'static str = stringify!($el);
                const NAME_SPACE: Option<&'static str> = None;
            }
        )*
    };
}

// A list of valid HTML5 elements (does not include removed or obsolete elements).
define_elements! {
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
    hgorup {},
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
    input {},
    ins {},
    kbd {},
    keygen {},
    label {},
    legend {},
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
    span {},
    strong {},
    style {},
    sub {},
    summary {},
    sup {},
    svg {},
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
    ul {},
    var {},
    video {},
    wbr {},
}

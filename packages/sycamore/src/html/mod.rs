//! HTML tag definitions.

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
            #[doc = concat!("Build a [`", stringify!($el), "`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/", stringify!($el), ") element.")]
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

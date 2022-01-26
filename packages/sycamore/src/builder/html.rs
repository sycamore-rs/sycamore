//! HTML-specific builder functions.

use crate::prelude::*;

macro_rules! generate_tag_functions {
    (
        $vis:vis enum $name:ident {
            $($tag:ident),* $(,)?
        }
    ) => {
        paste::paste! {
            $(
                pub fn [<$tag:lower>]<'a, G: GenericNode>(ctx: ScopeRef<'a>) -> crate::builder::agnostic::NodeBuilder<'a, G> {
                    crate::builder::agnostic::node(ctx, stringify!([<$tag:lower>]))
                }
            )*
        }
    };
}

// Source https://developer.mozilla.org/en-US/docs/Web/HTML/Element#demarcating_edits
generate_tag_functions! {
    enum HtmlTag {
        // Main Root
        Html,

        // Document markup
        Base,
        Head,
        Link,
        Meta,
        Style,
        Title,

        // Sectioning Root
        Body,

        // Content sectioning
        Address,
        Article,
        Aside,
        Footer,
        Header,
        H1,
        H2,
        H3,
        H4,
        H5,
        H6,
        Main,
        Nav,
        Section,

        // Text content
        Blockquote,
        Dd,
        Div,
        Dl,
        Dt,
        Figcaption,
        Figure,
        Hr,
        Li,
        Ol,
        P,
        Pre,
        Ul,

        // Inline text semantics
        A,
        Abbr,
        B,
        Bdi,
        Bdo,
        Br,
        Cite,
        Code,
        Data,
        Dfn,
        Em,
        I,
        Kbd,
        Mark,
        Q,
        Rp,
        Rt,
        Ruby,
        S,
        Samp,
        Small,
        Span,
        Strong,
        Sub,
        Sup,
        Time,
        U,
        Var,
        Wbr,

        // Image and multimedia
        Area,
        Audio,
        Img,
        Map,
        Track,
        Video,

        // Embeded content
        Embed,
        Iframe,
        Object,
        Param,
        Picture,
        Portal,
        Source,

        // SVG and MathML
        Svg,
        Math,

        // Scripting
        Canvas,
        Noscript,
        Script,

        // Demarcating edits
        Del,
        Ins,

        // Table content
        Caption,
        Col,
        Colgroup,
        Table,
        Tbody,
        Td,
        Tfoot,
        Th,
        Thead,
        Tr,

        // Forms
        Button,
        Datalist,
        Fieldset,
        Form,
        Input,
        Label,
        Legend,
        Meter,
        Optgroup,
        Option,
        Output,
        Progress,
        Select,
        Textarea,

        // Interactive elements
        Details,
        Dialog,
        Menu,
        Summary,

        // Web components
        Slot,
        Template,
    }
}

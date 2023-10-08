//! Intermediate representation for `view!` macro syntax.

use std::collections::HashSet;

use once_cell::sync::Lazy;
use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::{Expr, Ident, LitStr, Path, Token};

pub struct ViewRoot(pub Vec<ViewNode>);

pub enum ViewNode {
    Element(Element),
    Component(Component),
    Text(Text),
    Dyn(Dyn),
}

pub enum NodeType {
    Element,
    Component,
    Text,
    Dyn,
}
pub struct Element {
    pub tag: ElementTag,
    pub attrs: Vec<Attribute>,
    pub children: Vec<ViewNode>,
}

pub enum ElementTag {
    Builtin(Ident),
    Custom(String),
}

pub struct Attribute {
    pub ty: AttributeType,
    pub value: Expr,
    pub span: Span,
}

#[derive(PartialEq, Eq)]
pub enum AttributeType {
    /// An attribute that takes a value of a string.
    ///
    /// Syntax: `<name>`. `name` cannot be `dangerously_set_inner_html`.
    Str { name: String },
    /// An attribute that takes a value of a boolean.
    ///
    /// Syntax: `<name>`. `name` cannot be `dangerously_set_inner_html`.
    Bool { name: String },
    /// Syntax: `dangerously_set_inner_html`.
    DangerouslySetInnerHtml,
    /// Syntax: `on:<event>`.
    Event { event: Ident },
    /// Syntax: `bind:<prop>`.
    Bind { prop: String },
    /// Syntax: `prop:<prop>`.
    Property { prop: String },
    /// Syntax: `ref`.
    Ref,
    /// Syntax: ..attributes
    Spread,
}

pub fn is_bool_attr(name: &str) -> bool {
    // Boolean attributes list from the WHATWG attributes table:
    // https://html.spec.whatwg.org/multipage/indices.html#attributes-3
    static BOOLEAN_ATTRIBUTES_SET: Lazy<HashSet<&str>> = Lazy::new(|| {
        vec![
            "allowfullscreen",
            "async",
            "autofocus",
            "autoplay",
            "checked",
            "controls",
            "default",
            "defer",
            "disabled",
            "formnovalidate",
            "hidden",
            "inert",
            "ismap",
            "itemscope",
            "loop",
            "multiple",
            "muted",
            "nomodule",
            "novalidate",
            "open",
            "playsinline",
            "readonly",
            "required",
            "reversed",
            "selected",
        ]
        .into_iter()
        .collect()
    });
    BOOLEAN_ATTRIBUTES_SET.contains(name)
}

pub struct Component {
    pub ident: Path,
    pub props: Punctuated<ComponentProp, Token![,]>,
    pub brace: Option<Brace>,
    pub children: Option<ViewRoot>,
}

pub struct ComponentProp {
    pub prefix: Option<Ident>,
    pub name: String,
    pub eq: Token![=],
    pub value: Expr,
}

pub struct Text {
    pub value: LitStr,
}

pub struct Dyn {
    pub value: Expr,
}

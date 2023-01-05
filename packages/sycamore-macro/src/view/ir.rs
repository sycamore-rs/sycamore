//! Intermediate representation for `view!` macro syntax.

use std::collections::HashSet;

use once_cell::sync::Lazy;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
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
    pub name: Ident,
    pub eq: Token![=],
    pub value: Expr,
}

pub struct Text {
    pub value: LitStr,
}

pub struct Dyn {
    pub value: Expr,
}

fn needs_cx(ts: TokenStream, cx: &str) -> bool {
    for t in ts {
        match t {
            TokenTree::Ident(id) => {
                if id == cx {
                    return true;
                }
            }
            TokenTree::Group(g) => {
                if needs_cx(g.stream(), cx) {
                    return true;
                }
            }
            _ => (),
        }
    }
    false
}

impl Dyn {
    /// Returns `true` if the wrapped [`Expr`] has the identifier `cx` somewhere.
    pub fn needs_cx(&self, cx: &str) -> bool {
        let ts = self.value.to_token_stream();
        needs_cx(ts, cx)
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn needs_cx_if_cx_ident_inside_expr() {
        let ts: Dyn = parse_quote! {
            (create_signal(cx, 0))
        };
        assert!(ts.needs_cx("cx"));
        assert!(!ts.needs_cx("not_cx"));

        let not_cx: Dyn = parse_quote! {
            (123)
        };
        assert!(!not_cx.needs_cx("cx"));
        assert!(!not_cx.needs_cx("not_cx"));

        let ts_in_braces: Dyn = parse_quote! {
            ({
                create_signal(cx, 0)
            })
        };
        assert!(ts_in_braces.needs_cx("cx"));
        assert!(!ts_in_braces.needs_cx("not_cx"));
    }
}

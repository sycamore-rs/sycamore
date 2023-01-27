//! Intermediate representation for `view!` macro syntax.

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
    pub attrs: Punctuated<Attribute, Token![,]>,
    pub brace: Option<Brace>,
    pub children: Option<ViewRoot>,
}

pub enum ElementTag {
    Builtin(Ident),
    Custom(String),
}

#[derive(Clone)]
pub struct Attribute {
    pub ty: AttributeType,
    pub eq: Option<Token![=]>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq)]
pub enum AttributeType {
    /// Syntax: `ident`
    Ident(Ident),
    /// Syntax: `"custom-attribute"`
    Custom(LitStr),
    /// Syntax: `prefix:ident`
    PrefixedIdent(Ident, Token![:], Ident),
    /// Syntax: `prefix:"custom-attribute"`
    PrefixedCustom(Ident, Token![:], LitStr),
    /// Syntax: `..attributes`
    Spread,
}

impl Attribute {
    pub fn prefix(&self) -> Option<&Ident> {
        match &self.ty {
            AttributeType::PrefixedIdent(prefix, _, _) => Some(prefix),
            AttributeType::PrefixedCustom(prefix, _, _) => Some(prefix),
            _ => None,
        }
    }

    pub fn remove_prefix(&mut self) {
        self.ty = match &self.ty {
            AttributeType::PrefixedIdent(_, _, ident) => AttributeType::Ident(ident.clone()),
            AttributeType::PrefixedCustom(_, _, lit) => AttributeType::Custom(lit.clone()),
            ty => ty.clone(),
        };
    }
}

pub struct Component {
    pub ident: Path,
    pub props: Punctuated<Attribute, Token![,]>,
    pub brace: Option<Brace>,
    pub children: Option<ViewRoot>,
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

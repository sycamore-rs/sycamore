//! Intermediate representation for `view!` macro syntax.

use proc_macro2::Span;
use syn::{Expr, Ident, LitStr, Path};

pub struct Root(pub Vec<Node>);

pub enum Node {
    Tag(TagNode),
    Text(TextNode),
    Dyn(DynNode),
}

pub enum NodeType {
    Tag,
    Text,
    Dyn,
}

pub struct TagNode {
    pub ident: TagIdent,
    pub props: Vec<Prop>,
    pub children: Vec<Node>,
}

pub enum TagIdent {
    /// A standard Rust path.
    Path(Path),
    /// A hyphenated ident. Can not include any paths.
    /// This is used for custom elements support.
    Hyphenated(String),
}

pub struct Prop {
    pub ty: PropType,
    pub value: Expr,
    pub span: Span,
}

#[derive(PartialEq, Eq)]
pub enum PropType {
    /// Syntax: `<name>=<expr>`.
    Plain { ident: Ident },
    /// Syntax: `<dir>:<prop>=<expr>`.
    Directive { dir: Ident, ident: Ident },
    /// Syntax: `ref=<expr>`.
    Ref,
    /// Syntax: `..attributes=<expr>`
    Spread,
}

pub struct TextNode {
    pub value: LitStr,
}

pub struct DynNode {
    pub value: Expr,
}

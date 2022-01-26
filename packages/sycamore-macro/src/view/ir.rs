//! Intermediate representation for `view!` macro syntax.

use std::collections::HashSet;

use once_cell::sync::Lazy;
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, LitStr, Token};

pub struct ViewRoot(pub Vec<ViewNode>);

pub enum ViewNode {
    Element(Element),
    Component(Component),
    Text(Text),
    Dyn(Dyn),
}

impl ViewNode {
    /// Node is dynamic if the node is a component or a splice that is not a simple path.
    /// # Example
    /// ```ignore
    /// view! { MyComponent() } // is_dynamic = true
    /// view! { (state.get()) } // is_dynamic = true
    /// view! { (state) } // is_dynamic = false
    /// ```
    pub fn is_dynamic(&self) -> bool {
        match self {
            ViewNode::Element(_) => false,
            ViewNode::Component(_) => true,
            ViewNode::Text(_) => false,
            ViewNode::Dyn(Dyn {
                value: Expr::Lit(_) | Expr::Path(_),
            }) => false,
            ViewNode::Dyn(_) => true,
        }
    }
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
    Event { event: String },
    /// Syntax: `bind:<prop>`.
    Bind { prop: String },
    /// Syntax: `ref`.
    Ref,
}

pub fn is_bool_attr(name: &str) -> bool {
    static BOOLEAN_ATTRIBUTES_SET: Lazy<HashSet<&str>> = Lazy::new(|| {
        vec![
            "async",
            "autofocus",
            "autoplay",
            "border",
            "challenge",
            "checked",
            "compact",
            "contenteditable",
            "controls",
            "default",
            "defer",
            "disabled",
            "formNoValidate",
            "frameborder",
            "hidden",
            "indeterminate",
            "ismap",
            "loop",
            "multiple",
            "muted",
            "nohref",
            "noresize",
            "noshade",
            "novalidate",
            "nowrap",
            "open",
            "readonly",
            "required",
            "reversed",
            "scoped",
            "scrolling",
            "seamless",
            "selected",
            "sortable",
            "spellcheck",
            "translate",
        ]
        .into_iter()
        .collect()
    });
    BOOLEAN_ATTRIBUTES_SET.contains(name)
}

pub enum Component {
    FnLike(FnLikeComponent),
    ElementLike(ElementLikeComponent),
}

pub struct FnLikeComponent {
    pub ident: Ident,
    pub args: Punctuated<Expr, Token![,]>,
}

pub struct ElementLikeComponent {
    pub ident: Ident,
    pub props: Vec<(Ident, Expr)>,
}

pub struct Text {
    pub value: LitStr,
}

pub struct Dyn {
    pub value: Expr,
}

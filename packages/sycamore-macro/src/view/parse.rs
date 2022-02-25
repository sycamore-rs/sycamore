//! Parse syntax for `view!` macro.

use std::fmt;

use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Paren};
use syn::{braced, parenthesized, token, Expr, FieldValue, Ident, LitStr, Result, Token};

use super::ir::*;

impl Parse for ViewRoot {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children = Vec::new();

        while !input.is_empty() {
            children.push(input.parse()?);
        }

        Ok(Self(children))
    }
}

impl ViewNode {
    fn peek_type(input: ParseStream) -> Option<NodeType> {
        let input = input.fork(); // do not affect original ParseStream

        if input.peek(LitStr) {
            Some(NodeType::Text)
        } else if input.peek(Paren) {
            Some(NodeType::Dyn)
        } else if input.peek(Token![::]) {
            Some(NodeType::Component)
        } else if input.peek(Ident::peek_any) {
            let ident: Ident = input.call(Ident::parse_any).ok()?;
            let ident = ident.to_string();

            if ident.chars().next().unwrap().is_ascii_uppercase() || input.peek(Token![::]) {
                Some(NodeType::Component)
            } else {
                Some(NodeType::Element)
            }
        } else {
            None
        }
    }
}

impl Parse for ViewNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = match Self::peek_type(input) {
            Some(ty) => ty,
            None => return Err(input.error("expected a valid node")),
        };

        Ok(match ty {
            NodeType::Element => Self::Element(input.parse()?),
            NodeType::Component => Self::Component(input.parse()?),
            NodeType::Text => Self::Text(input.parse()?),
            NodeType::Dyn => Self::Dyn(input.parse()?),
        })
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag = input.parse()?;
        let attrs = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            content
                .parse_terminated::<Attribute, Token![,]>(Attribute::parse)?
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };
        let children = if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let mut body = Vec::new();
            while !content.is_empty() {
                body.push(content.parse()?);
            }
            body
        } else {
            Vec::new()
        };

        let has_dangerously_set_inner_html_attr = attrs
            .iter()
            .any(|attr| attr.ty == AttributeType::DangerouslySetInnerHtml);
        if has_dangerously_set_inner_html_attr && !children.is_empty() {
            return Err(input.error("children and dangerously_set_inner_html cannot be both set"));
        }

        Ok(Self {
            tag,
            attrs,
            children,
        })
    }
}

impl Parse for ElementTag {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag = input.call(Ident::parse_any)?;
        let mut extended = Vec::<(Token![-], Ident)>::new();
        while input.peek(Token![-]) {
            extended.push((input.parse()?, input.parse()?));
        }
        if extended.is_empty() {
            Ok(Self::Builtin(tag))
        } else {
            let tag = format!(
                "{tag}-{extended}",
                extended = extended
                    .into_iter()
                    .map(|x| x.1.to_string())
                    .collect::<Vec<_>>()
                    .join("-")
            );
            Ok(Self::Custom(tag))
        }
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse()?;
        let _eqs: Token![=] = input.parse()?;
        let value = input.parse()?;
        Ok(Self { ty, value })
    }
}

impl Parse for AttributeType {
    fn parse(input: ParseStream) -> Result<Self> {
        pub struct AttributeName {
            tag: Ident,
            extended: Vec<(Token![-], Ident)>,
        }

        impl Parse for AttributeName {
            fn parse(input: ParseStream) -> Result<Self> {
                let tag = input.call(Ident::parse_any)?;
                let mut extended = Vec::new();
                while input.peek(Token![-]) {
                    extended.push((input.parse()?, input.parse()?));
                }

                Ok(Self { tag, extended })
            }
        }

        impl fmt::Display for AttributeName {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let AttributeName { tag, extended } = self;

                write!(f, "{}", tag)?;
                for (_, ident) in extended {
                    write!(f, "-{}", ident)?;
                }

                Ok(())
            }
        }

        let ident: AttributeName = input.parse()?;
        let name = ident.to_string();

        if name == "ref" {
            Ok(Self::Ref)
        } else if name == "dangerously_set_inner_html" {
            Ok(Self::DangerouslySetInnerHtml)
        } else if input.peek(Token![:]) {
            let _colon: Token![:] = input.parse()?;
            match name.as_str() {
                "on" => {
                    let event = input.call(Ident::parse_any)?;
                    Ok(Self::Event {
                        event: event.to_string(),
                    })
                }
                "bind" => {
                    let prop = input.call(Ident::parse_any)?;
                    Ok(Self::Bind {
                        prop: prop.to_string(),
                    })
                }
                _ => Err(syn::Error::new_spanned(
                    ident.tag,
                    format!("unknown directive `{}`", name),
                )),
            }
        } else if is_bool_attr(&name) {
            Ok(Self::Bool { name })
        } else {
            Ok(Self::Str { name })
        }
    }
}

impl Parse for Component {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let content;
        if input.peek(Paren) {
            // Parse fn-like component.
            parenthesized!(content in input);
            let args = content.parse_terminated(Expr::parse)?;
            Ok(Self::FnLike(FnLikeComponent { ident, args }))
        } else if input.peek(Brace) {
            // Parse element link component.
            braced!(content in input);
            let mut props = Punctuated::<FieldValue, Token![,]>::new();
            while !content.is_empty() {
                if content.peek(Ident) && content.peek2(Token![:]) && !content.peek3(Token![:]) {
                    // Parse component prop field.
                    let field_value = content.parse()?;
                    let comma_parsed = if content.peek(Token![,]) {
                        let _comma: Token![,] = content.parse()?;
                        true
                    } else {
                        false
                    };
                    if !content.is_empty() && !comma_parsed {
                        content.parse::<Token![,]>()?; // Emit an error if there is no comma and not
                                                       // eof.
                    }
                    props.push(field_value);
                } else {
                    break;
                }
            }
            let children = if content.peek(Brace) {
                // Parse view fragment as children
                let children;
                braced!(children in content);
                Some(ViewRoot::parse(&children)?)
            } else if ViewNode::peek_type(&content).is_some() {
                Some(ViewRoot(vec![ViewNode::parse(&content)?]))
            } else {
                None
            };
            Ok(Self::ElementLike(ElementLikeComponent {
                ident,
                props: props
                    .into_iter()
                    .map(|x| match x.member {
                        syn::Member::Named(named) => (named, x.expr),
                        syn::Member::Unnamed(_) => todo!("implement error handling"),
                    })
                    .collect(),
                children,
            }))
        } else {
            Err(input.error("expected either `(` or `{`"))
        }
    }
}

impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            value: input.parse()?,
        })
    }
}

impl Parse for Dyn {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(Self {
            value: content.parse()?,
        })
    }
}

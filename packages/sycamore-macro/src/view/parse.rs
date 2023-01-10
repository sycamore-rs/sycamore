//! Parse syntax for `view!` macro.

use std::fmt;

use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Paren};
use syn::{braced, parenthesized, token, Ident, LitStr, Result, Token};

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
        let mut has_paren = false;
        let attrs = if input.peek(token::Paren) {
            has_paren = true;
            let content;
            parenthesized!(content in input);
            content
                .parse_terminated::<Attribute, Token![,]>(Attribute::parse)?
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };
        if !has_paren && !input.peek(Brace) {
            return Err(input.error("expected either `(` or `{` after element tag"));
        }
        let mut children = Vec::new();
        if input.peek(Brace) {
            let content;
            braced!(content in input);
            while !content.is_empty() {
                children.push(content.parse()?);
            }
        }
        // Check if dangerously_set_inner_html is also set.
        let dangerously_set_inner_html_span = attrs.iter().find_map(|attr| {
            (attr.ty == AttributeType::DangerouslySetInnerHtml).then_some(attr.span)
        });
        if let Some(span) = dangerously_set_inner_html_span {
            if !children.is_empty() {
                return Err(syn::Error::new(
                    span,
                    "children and dangerously_set_inner_html cannot be both set",
                ));
            }
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
        let span = input.span();
        let ty = input.parse()?;
        if !matches!(ty, AttributeType::Spread { .. }) {
            let _eqs: Token![=] = input.parse()?;
        }
        let value = input.parse()?;
        Ok(Self { ty, value, span })
    }
}

impl Parse for AttributeType {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![..]) {
            let _2dot = input.parse::<Token![..]>()?;
            Ok(Self::Spread)
        } else if lookahead.peek(Ident::peek_any) {
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
                        Ok(Self::Event { event })
                    }
                    "prop" => {
                        let prop = input.call(Ident::parse_any)?;
                        Ok(Self::Property {
                            prop: prop.to_string(),
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
        } else {
            Err(lookahead.error())
        }
    }
}

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

impl Parse for Component {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;

        let mut props = Punctuated::new();
        let mut brace = None;
        let mut children = None;

        if input.peek(Paren) {
            // Parse props.
            let content;
            parenthesized!(content in input);
            props = content.parse_terminated(ComponentProp::parse)?;
        }
        if input.peek(Brace) {
            // Parse children.
            let content;
            brace = Some(braced!(content in input));
            children = Some(content.parse()?);
        }

        Ok(Self {
            ident,
            props,
            brace,
            children,
        })
    }
}

impl Parse for ComponentProp {
    fn parse(input: ParseStream) -> Result<Self> {
        let name_or_prefix: Ident = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![:]) {
            let _colon = input.parse::<Token![:]>()?;
            let name: AttributeName = input.parse()?;
            Ok(Self {
                prefix: Some(name_or_prefix),
                name: name.to_string(),
                eq: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Ok(Self {
                prefix: None,
                name: name_or_prefix.to_string(),
                eq: input.parse()?,
                value: input.parse()?,
            })
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

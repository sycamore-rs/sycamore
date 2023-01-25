//! Parse syntax for `view!` macro.

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
        let lookahead = input.lookahead1();

        let mut has_paren = false;
        let attrs = if input.peek(token::Paren) {
            has_paren = true;
            let content;
            parenthesized!(content in input);
            content.parse_terminated::<Attribute, Token![,]>(Attribute::parse)?
        } else {
            Punctuated::new()
        };

        if !has_paren && !lookahead.peek(Brace) {
            return Err(lookahead.error());
        }

        let mut brace = None;
        let mut children: Option<ViewRoot> = None;
        if input.peek(Brace) {
            // Parse children.
            let content;
            brace = Some(braced!(content in input));
            children = Some(content.parse()?);
        }

        // Check if dangerously_set_inner_html is also set. If so, get the span.
        let dangerously_set_inner_html_span = attrs.iter().find(|attr| {
            matches!(&attr.ty, AttributeType::Ident(ident) if ident == "dangerously_set_inner_html")
        }).map(|attr| attr.span);

        if let (Some(span), Some(children)) = (dangerously_set_inner_html_span, &children) {
            if children.0.is_empty() {
                return Err(syn::Error::new(
                    span,
                    "children and dangerously_set_inner_html cannot be both set",
                ));
            }
        }

        Ok(Self {
            tag,
            attrs,
            brace,
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
        let mut eq = None;
        if !matches!(ty, AttributeType::Spread { .. }) {
            eq = input.parse()?;
        }
        let value = input.parse()?;
        Ok(Self {
            ty,
            eq,
            value,
            span,
        })
    }
}

impl Parse for AttributeType {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![..]) {
            let _2dot = input.parse::<Token![..]>()?;
            Ok(Self::Spread)
        } else if lookahead.peek(Ident::peek_any) {
            let ident = input.parse()?;
            // Check if this ident is a prefix or not.
            if input.peek(Token![:]) {
                let prefix = ident;
                let colon = input.parse()?;
                let lookahead2 = input.lookahead1();
                if lookahead2.peek(Ident::peek_any) {
                    let ident = input.parse()?;
                    Ok(Self::PrefixedIdent(prefix, colon, ident))
                } else if lookahead2.peek(LitStr) {
                    let custom = input.parse()?;
                    Ok(Self::PrefixedCustom(prefix, colon, custom))
                } else {
                    Err(lookahead2.error())
                }
            } else {
                Ok(Self::Ident(ident))
            }
        } else if lookahead.peek(LitStr) {
            let custom = input.parse()?;
            Ok(Self::Custom(custom))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Component {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;

        let lookahead = input.lookahead1();

        let mut has_paren = false;
        let props = if input.peek(token::Paren) {
            has_paren = true;
            let content;
            parenthesized!(content in input);
            content.parse_terminated::<Attribute, Token![,]>(Attribute::parse)?
        } else {
            Punctuated::new()
        };

        if !has_paren && !lookahead.peek(Brace) {
            return Err(lookahead.error());
        }

        let mut brace = None;
        let mut children: Option<ViewRoot> = None;
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

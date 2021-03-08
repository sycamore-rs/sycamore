use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{parenthesized, Expr, Ident, Result, Token};

pub enum AttributeType {
    /// Syntax: `name`.
    DomAttribute { name: String },
    /// Syntax: `on:name`.
    Event { name: String },
}

impl Parse for AttributeType {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.call(Ident::parse_any)?;
        let ident_str = ident.to_string();

        if input.peek(Token![:]) {
            let _colon: Token![:] = input.parse()?;
            match ident_str.as_str() {
                "on" => {
                    let event_name = input.call(Ident::parse_any)?;
                    Ok(Self::Event {
                        name: event_name.to_string(),
                    })
                }
                _ => Err(syn::Error::new_spanned(
                    ident,
                    format!("unknown directive `{}`", ident_str),
                )),
            }
        } else {
            Ok(Self::DomAttribute { name: ident_str })
        }
    }
}

pub struct Attribute {
    pub ty: AttributeType,
    pub equals_token: Token![=],
    pub expr: Expr,
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            ty: input.parse()?,
            equals_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

pub struct AttributeList {
    pub paren_token: Paren,
    pub attributes: Punctuated<Attribute, Token![,]>,
}

impl Parse for AttributeList {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);

        let attributes = content.parse_terminated(Attribute::parse)?;

        Ok(Self {
            paren_token,
            attributes,
        })
    }
}

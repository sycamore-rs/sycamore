use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Expr, ExprAssign, ExprPath, Ident, Token};

use crate::children::Children;

/// Represents a html element with all its attributes and properties (e.g. `p(class="text")`).
pub(crate) struct HtmlElement {
    tag_name: TagName,
    _paren_token: token::Paren,
    attributes: Punctuated<Expr, Token![,]>,
    children: Option<Children>,
}

impl Parse for HtmlElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag_name = input.parse()?;
        let attributes;
        let paren_token = parenthesized!(attributes in input);
        let attributes = attributes.parse_terminated(Expr::parse)?;

        let children = if input.peek(token::Brace) {
            Some(input.parse()?)
        } else {
            None
        };

        // check attribute syntax
        for attribute in &attributes {
            match attribute {
                Expr::Assign(ExprAssign {
                    attrs: _,
                    left,
                    eq_token: _,
                    right: _,
                }) => {
                    if !matches!(left.as_ref(), Expr::Path(ExprPath {path, ..}) if path.segments.len() == 1)
                    {
                        return Err(syn::Error::new_spanned(left, "expected an identifier"));
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        attribute,
                        "expected an assignment expression",
                    ))
                }
            }
        }

        Ok(Self {
            tag_name,
            _paren_token: paren_token,
            attributes,
            children,
        })
    }
}

impl ToTokens for HtmlElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let HtmlElement {
            tag_name,
            _paren_token: _,
            attributes,
            children,
        } = self;

        let mut set_attributes = Vec::new();
        for attribute in attributes {
            match attribute {
                Expr::Assign(ExprAssign {
                    attrs: _,
                    left,
                    eq_token: _,
                    right,
                }) => {
                    let left_str = left.to_token_stream().to_string();

                    set_attributes.push(quote! {
                        ::maple_core::internal::attr(&element, #left_str, #right);
                    });
                }
                _ => unreachable!("attribute syntax checked during parsing"),
            }
        }

        let mut append_children = Vec::new();
        if let Some(children) = children {
            for child in &children.body {
                append_children.push(quote! {
                    ::maple_core::internal::append(&element, #child);
                });
            }
        }

        let quoted = quote! {{
            let element = #tag_name;
            #(#set_attributes)*
            #(#append_children)*
            element
        }};
        tokens.extend(quoted);
    }
}

/// Represents a html element tag (e.g. `div`, `custom-element` etc...).
pub struct TagName {
    tag: Ident,
    extended: Vec<(Token![-], Ident)>,
}

impl Parse for TagName {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag = input.call(Ident::parse_any)?;
        let mut extended = Vec::new();
        while input.peek(Token![-]) {
            extended.push((input.parse()?, input.parse()?));
        }

        Ok(Self { tag, extended })
    }
}

impl ToTokens for TagName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TagName { tag, extended } = self;

        let mut tag_str = tag.to_string();
        for (_, ident) in extended {
            tag_str.push_str(&format!("-{}", ident));
        }

        let quoted = quote! {
            ::maple_core::internal::element(#tag_str)
        };

        tokens.extend(quoted);
    }
}

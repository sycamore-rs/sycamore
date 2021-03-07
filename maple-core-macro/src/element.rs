use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parenthesized, token, Expr, ExprAssign, ExprPath, ExprType, Ident, Token, Type, TypePath,
};

use crate::children::Children;

/// Represents a html element with all its attributes and properties (e.g. `p(class="text")`).
pub(crate) struct HtmlElement {
    tag_name: TagName,
    _paren_token: Option<token::Paren>,
    attributes: Punctuated<Expr, Token![,]>,
    children: Option<Children>,
}

impl Parse for HtmlElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag_name = input.parse()?;
        let (paren_token, attributes) = if input.peek(token::Paren) {
            let attributes;
            let paren_token = parenthesized!(attributes in input);
            (Some(paren_token), attributes.parse_terminated(Expr::parse)?)
        } else {
            (None, Punctuated::new())
        };

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
                    match left.as_ref() {
                        // simple attribute (e.g. `disabled`)
                        Expr::Path(ExprPath { path, .. }) if path.segments.len() == 1 => {}
                        // `on:click` is parsed as a type ascription expression
                        Expr::Type(ExprType {
                            attrs: _,
                            expr,
                            colon_token: _,
                            ty,
                        }) => match expr.as_ref() {
                            Expr::Path(ExprPath { path, .. }) if path.segments.len() == 1 => {
                                match path.segments[0].ident.to_string().as_str() {
                                    "on" => {}
                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            &path.segments[0],
                                            format!(
                                                "unknown directive `{}`",
                                                path.segments[0].ident
                                            ),
                                        ))
                                    }
                                }

                                match ty.as_ref() {
                                    Type::Path(TypePath { path, .. })
                                        if path.segments.len() == 1 => {}
                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            ty,
                                            "expected an identifier",
                                        ))
                                    }
                                }
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(expr, "expected an identifier"))
                            }
                        },
                        _ => return Err(syn::Error::new_spanned(left, "unexpected token")),
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
        let mut set_event_listeners = Vec::new();
        for attribute in attributes {
            let attribute_span = attribute.span();

            match attribute {
                Expr::Assign(ExprAssign {
                    attrs: _,
                    left,
                    eq_token: _,
                    right,
                }) => match left.as_ref() {
                    Expr::Path(_) => {
                        let left_str = left.to_token_stream().to_string();

                        set_attributes.push(quote_spanned! { attribute_span=>
                            ::maple_core::internal::attr(&element, #left_str, move || ::std::format!("{}", #right));
                        });
                    }
                    Expr::Type(ExprType {
                        attrs: _,
                        expr,
                        colon_token: _,
                        ty,
                    }) => match expr.as_ref() {
                        Expr::Path(path) => {
                            let directive = path.to_token_stream().to_string();

                            match directive.as_str() {
                                "on" => {
                                    // attach event handler
                                    let event_name = ty.to_token_stream().to_string();

                                    set_event_listeners.push(quote_spanned! { attribute_span=>
                                        ::maple_core::internal::event(&element, #event_name, ::std::boxed::Box::new(#right));
                                    });
                                }
                                _ => unreachable!("attribute syntax checked during parsing"),
                            }
                        }
                        _ => unreachable!("attribute syntax checked during parsing"),
                    },
                    _ => unreachable!("attribute syntax checked during parsing"),
                },
                _ => unreachable!("attribute syntax checked during parsing"),
            }
        }

        let mut append_children = Vec::new();
        if let Some(children) = children {
            for child in &children.body {
                append_children.push(quote! {
                    ::maple_core::internal::append(&element, &&#child);
                });
            }
        }

        let quoted = quote! {{
            let element = #tag_name;
            #(#set_attributes)*
            #(#set_event_listeners)*
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

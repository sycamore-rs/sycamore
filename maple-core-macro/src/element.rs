use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Expr, ExprAssign, Ident, Token};

/// Represents a html element with all its attributes and properties (e.g. `p(class="text")`).
pub struct HtmlElement {
    tag_name: TagName,
    _paren_token: token::Paren,
    attributes: Punctuated<Expr, Token![,]>,
}

impl Parse for HtmlElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes;
        Ok(Self {
            tag_name: input.parse()?,
            _paren_token: parenthesized!(attributes in input),
            attributes: attributes.parse_terminated(Expr::parse)?,
        })
    }
}

impl ToTokens for HtmlElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let HtmlElement {
            tag_name,
            _paren_token: _,
            attributes,
        } = self;

        let mut set_attributes = Vec::new();
        for attribute in attributes {
            match attribute {
                Expr::Assign(expr) => {
                    let ExprAssign {
                        attrs: _,
                        left,
                        eq_token: _,
                        right,
                    } = expr;

                    let left_str = left.to_token_stream().to_string();

                    set_attributes.push(quote! {
                        ::maple_core::internal::attr(&element, #left_str, #right);
                    });
                }
                _ => unreachable!(),
            }
        }

        let quoted = quote! {{
            let element = #tag_name;
            #(#set_attributes);*
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

use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{token, Ident, Token};

use super::*;

/// Represents a html element with all its attributes and properties (e.g. `p(class="text")`).
pub struct Element {
    pub tag_name: TagName,
    pub attributes: Option<AttributeList>,
    pub children: Option<Children>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag_name = input.parse()?;

        let attributes = if input.peek(token::Paren) {
            Some(input.parse()?)
        } else {
            None
        };

        let children = if input.peek(token::Brace) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            tag_name,
            attributes,
            children,
        })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Element {
            tag_name,
            attributes,
            children,
        } = self;

        let mut quoted = quote! {
            let _el = #tag_name;
        };

        if let Some(attributes) = attributes {
            for attribute in &attributes.attributes {
                attribute.to_tokens(&mut quoted);
            }
        }

        if let Some(children) = children {
            for child in &children.body {
                quoted.extend(match child {
                    HtmlTree::Component(component) => quote_spanned! { component.span()=>
                        let __marker = ::sycamore::generic_node::GenericNode::marker();
                        ::sycamore::generic_node::GenericNode::append_child(&_el, &__marker);
                        ::sycamore::generic_node::render::insert(
                            ::std::clone::Clone::clone(&_el),
                            #component,
                            None, Some(__marker),
                        );
                    },
                    HtmlTree::Element(element) => quote_spanned! { element.span()=>
                        ::sycamore::generic_node::GenericNode::append_child(&_el, &#element);
                    },
                    HtmlTree::Text(text) => match text {
                        Text::Str(_) => {
                            quote_spanned! { text.span()=>
                                ::sycamore::generic_node::GenericNode::append_child(
                                    &_el,
                                    &::sycamore::generic_node::GenericNode::text_node(#text),
                                );
                            }
                        }
                        Text::Splice(_, _) => {
                            quote_spanned! { text.span()=>
                                let __marker = ::sycamore::generic_node::GenericNode::marker();
                                ::sycamore::generic_node::GenericNode::append_child(&_el, &__marker);
                                ::sycamore::generic_node::render::insert(
                                    ::std::clone::Clone::clone(&_el),
                                    ::sycamore::template::Template::new_lazy(move ||
                                        ::sycamore::render::IntoTemplate::create(&#text)
                                    ),
                                    None, Some(__marker),
                                );
                            }
                        }
                    },
                });
            }
        }

        quoted.extend(quote! {
            _el
        });
        tokens.extend(quote! {{
            #quoted
        }});
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
        let tag_str = self.to_string();

        let quoted = quote! {
            ::sycamore::generic_node::GenericNode::element(#tag_str)
        };

        tokens.extend(quoted);
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let TagName { tag, extended } = self;

        write!(f, "{}", tag)?;
        for (_, ident) in extended {
            write!(f, "-{}", ident)?;
        }

        Ok(())
    }
}

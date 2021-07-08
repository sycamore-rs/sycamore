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
            let __el = #tag_name;
        };

        if let Some(attributes) = attributes {
            for attribute in &attributes.attributes {
                attribute.to_tokens(&mut quoted);
            }
        }

        if let Some(children) = children {
            let multi = children.body.len() != 1;
            let mut children = children.body.iter().peekable();
            while let Some(child) = children.next() {
                quoted.extend(match child {
                    HtmlTree::Component(_) | HtmlTree::Text(Text::Splice(..)) => {
                        let quote_marker =
                        if let Some(HtmlTree::Element(element)) =
                            children.next_if(|x| matches!(x, HtmlTree::Element(_)))
                        {
                            quote_spanned! { element.span()=>
                                let __marker = #element;
                                ::sycamore::generic_node::GenericNode::append_child(&__el, &__marker);
                                let __marker = ::std::option::Option::Some(&__marker);
                            }
                        } else if let Some(HtmlTree::Text(Text::Str(text))) =
                            children.next_if(|x| matches!(x, HtmlTree::Text(Text::Str(_))))
                        {
                            quote_spanned! { text.span()=>
                                let __marker = ::sycamore::generic_node::GenericNode::text_node(#text);
                                ::sycamore::generic_node::GenericNode::append_child(&__el, &__marker);
                                let __marker = ::std::option::Option::Some(&__marker);
                            }
                        } else if children.peek().is_none() {
                            quote! {
                                let __marker = ::std::option::Option::None;
                            }
                        } else {
                            quote! {
                                let __marker = ::sycamore::generic_node::GenericNode::marker();
                                ::sycamore::generic_node::GenericNode::append_child(&__el, &__marker);
                                let __marker = ::std::option::Option::Some(&__marker);
                            }
                        };
                        match child {
                            HtmlTree::Component(component) => quote_spanned! { component.span()=>
                                #quote_marker
                                ::sycamore::utils::render::insert(
                                    &__el,
                                    #component,
                                    None, __marker, #multi
                                );
                            },
                            HtmlTree::Text(text @ Text::Splice(..)) => quote_spanned! { text.span()=>
                                #quote_marker
                                ::sycamore::utils::render::insert(
                                   &__el,
                                   ::sycamore::template::Template::new_dyn(move ||
                                       ::sycamore::template::IntoTemplate::create(#text)
                                   ),
                                   None, __marker, #multi
                               );
                            },
                            _ => unreachable!()
                        }
                    }
                    HtmlTree::Element(element) => quote_spanned! { element.span()=>
                        ::sycamore::generic_node::GenericNode::append_child(&__el, &#element);
                    },
                    HtmlTree::Text(text @ Text::Str(_)) => quote_spanned! { text.span()=>
                        ::sycamore::generic_node::GenericNode::append_child(
                            &__el,
                            &::sycamore::generic_node::GenericNode::text_node(#text),
                        );
                    },
                });
            }
        }

        quoted.extend(quote! {
            __el
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

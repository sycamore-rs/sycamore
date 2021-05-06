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

        let mut set_attributes = Vec::new();
        let mut set_event_listeners = Vec::new();
        let mut set_noderefs = Vec::new();
        if let Some(attributes) = attributes {
            for attribute in &attributes.attributes {
                let expr = &attribute.expr;
                let expr_span = expr.span();

                match &attribute.ty {
                    AttributeType::DomAttribute { name } => {
                        let name = name.to_string();
                        set_attributes.push(quote_spanned! { expr_span=>
                            ::maple_core::reactive::create_effect({
                                let element = ::std::clone::Clone::clone(&element);
                                move || {
                                    ::maple_core::generic_node::GenericNode::set_attribute(
                                        &element,
                                        #name,
                                        &::std::format!("{}", #expr),
                                    );
                                }
                            });
                        });
                    }
                    AttributeType::Event { event } => {
                        // TODO: Should events be reactive?
                        set_event_listeners.push(quote_spanned! { expr_span=>
                            ::maple_core::generic_node::GenericNode::event(
                                &element,
                                #event,
                                ::std::boxed::Box::new(#expr),
                            );
                        });
                    }
                    AttributeType::Bind { prop } => {
                        #[derive(Clone, Copy)]
                        enum JsPropertyType {
                            Bool,
                            String,
                        }

                        let (event_name, property_ty) = match prop.as_str() {
                            "value" => ("input", JsPropertyType::String),
                            "checked" => ("change", JsPropertyType::Bool),
                            _ => {
                                tokens.extend(
                                    syn::Error::new(
                                        prop.span(),
                                        &format!("property `{}` is not supported with bind:", prop),
                                    )
                                    .to_compile_error(),
                                );
                                return;
                            }
                        };

                        let value_ty = match property_ty {
                            JsPropertyType::Bool => quote! { ::std::primitive::bool },
                            JsPropertyType::String => quote! { ::std::string::String },
                        };

                        let convert_into_jsvalue_fn = match property_ty {
                            JsPropertyType::Bool => {
                                quote! { ::maple_core::rt::JsValue::from_bool(*signal.get()) }
                            }
                            JsPropertyType::String => {
                                quote! { ::maple_core::rt::JsValue::from_str(&::std::format!("{}", signal.get())) }
                            }
                        };

                        let event_target_prop = quote! {
                            ::maple_core::rt::Reflect::get(
                                &event.target().unwrap(),
                                &::std::convert::Into::<::maple_core::rt::JsValue>::into(#prop)
                            ).unwrap()
                        };

                        let convert_from_jsvalue_fn = match property_ty {
                            JsPropertyType::Bool => quote! {
                                ::maple_core::rt::JsValue::as_bool(&#event_target_prop).unwrap()
                            },
                            JsPropertyType::String => quote! {
                                ::maple_core::rt::JsValue::as_string(&#event_target_prop).unwrap()
                            },
                        };

                        set_attributes.push(quote_spanned! { expr_span=> {
                            let signal: ::maple_core::reactive::Signal<#value_ty> = #expr;

                            ::maple_core::reactive::create_effect({
                                let signal = ::std::clone::Clone::clone(&signal);
                                let element = ::std::clone::Clone::clone(&element);
                                move || {
                                    ::maple_core::generic_node::GenericNode::set_property(
                                        &element,
                                        #prop,
                                        &#convert_into_jsvalue_fn,
                                    );
                                }
                            });

                            ::maple_core::generic_node::GenericNode::event(
                                &element,
                                #event_name,
                                ::std::boxed::Box::new(move |event: ::maple_core::rt::Event| {
                                    signal.set(#convert_from_jsvalue_fn);
                                }),
                            )
                        }});
                    }
                    AttributeType::Ref => {
                        set_noderefs.push(quote_spanned! { expr_span=>
                            ::maple_core::noderef::NodeRef::set(
                                &#expr,
                                ::std::clone::Clone::clone(&element),
                            );
                        });
                    }
                }
            }
        }

        let mut append_children = Vec::new();
        if let Some(children) = children {
            for child in &children.body {
                let quoted = match child {
                    HtmlTree::Component(component) => quote_spanned! { component.span()=>
                        ::maple_core::generic_node::render::insert(
                            ::std::clone::Clone::clone(&element),
                            #component,
                            None, None,
                        );
                    },
                    HtmlTree::Element(element) => quote_spanned! { element.span()=>
                        ::maple_core::generic_node::GenericNode::append_child(&element, &#element);
                    },
                    HtmlTree::Text(text) => match text {
                        Text::Text(_) => {
                            quote_spanned! { text.span()=>
                                ::maple_core::generic_node::GenericNode::append_child(
                                    &element,
                                    &::maple_core::generic_node::GenericNode::text_node(#text),
                                );
                            }
                        }
                        Text::Splice(_, _) => {
                            quote_spanned! { text.span()=>
                                ::maple_core::generic_node::render::insert(
                                    ::std::clone::Clone::clone(&element),
                                    ::maple_core::template_result::TemplateResult::new_lazy(move || {
                                        let mut nodes = ::maple_core::render::Render::create(&#text);
                                        if ::std::vec::Vec::len(&nodes) == 1 {
                                            ::maple_core::template_result::TemplateResult::new_node(
                                                ::std::vec::Vec::remove(&mut nodes, 0)
                                            )
                                        } else {
                                            let nodes = nodes.into_iter().map(::maple_core::template_result::TemplateResult::new_node).collect();
                                            ::maple_core::template_result::TemplateResult::new_fragment(nodes)
                                        }
                                    }),
                                    None, None,
                                );
                            }
                        }
                    },
                };

                append_children.push(quoted);
            }
        }

        let quoted = quote! {{
            let element = #tag_name;
            #(#set_attributes)*
            #(#set_event_listeners)*
            #(#set_noderefs)*
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
        let tag_str = self.to_string();

        let quoted = quote! {
            ::maple_core::generic_node::GenericNode::element(#tag_str)
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

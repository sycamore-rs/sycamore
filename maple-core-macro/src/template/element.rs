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
                let expr = &attribute.expr;
                let expr_span = expr.span();

                match &attribute.ty {
                    AttributeType::DomAttribute { name } => {
                        let name = name.to_string();
                        quoted.extend(quote_spanned! { expr_span=>
                            ::maple_core::reactive::create_effect({
                                let _el = ::std::clone::Clone::clone(&_el);
                                move || {
                                    ::maple_core::generic_node::GenericNode::set_attribute(
                                        &_el,
                                        #name,
                                        &::std::format!("{}", #expr),
                                    );
                                }
                            });
                        });
                    }
                    AttributeType::Event { event } => {
                        // TODO: Should events be reactive?
                        quoted.extend(quote_spanned! { expr_span=>
                            ::maple_core::generic_node::GenericNode::event(
                                &_el,
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

                        quoted.extend(quote_spanned! { expr_span=> {
                            let signal: ::maple_core::reactive::Signal<#value_ty> = #expr;

                            ::maple_core::reactive::create_effect({
                                let signal = ::std::clone::Clone::clone(&signal);
                                let _el = ::std::clone::Clone::clone(&_el);
                                move || {
                                    ::maple_core::generic_node::GenericNode::set_property(
                                        &_el,
                                        #prop,
                                        &#convert_into_jsvalue_fn,
                                    );
                                }
                            });

                            ::maple_core::generic_node::GenericNode::event(
                                &_el,
                                #event_name,
                                ::std::boxed::Box::new(move |event: ::maple_core::rt::Event| {
                                    signal.set(#convert_from_jsvalue_fn);
                                }),
                            )
                        }});
                    }
                    AttributeType::Ref => {
                        quoted.extend(quote_spanned! { expr_span=>{
                            ::maple_core::noderef::NodeRef::set(
                                &#expr,
                                ::std::clone::Clone::clone(&_el),
                            );
                        }});
                    }
                }
            }
        }

        if let Some(children) = children {
            for child in &children.body {
                quoted.extend(match child {
                    HtmlTree::Component(component) => quote_spanned! { component.span()=>
                        ::maple_core::generic_node::render::insert(
                            ::std::clone::Clone::clone(&_el),
                            #component,
                            None, None,
                        );
                    },
                    HtmlTree::Element(element) => quote_spanned! { element.span()=>
                        ::maple_core::generic_node::GenericNode::append_child(&_el, &#element);
                    },
                    HtmlTree::Text(text) => match text {
                        Text::Text(_) => {
                            quote_spanned! { text.span()=>
                                ::maple_core::generic_node::GenericNode::append_child(
                                    &_el,
                                    &::maple_core::generic_node::GenericNode::text_node(#text),
                                );
                            }
                        }
                        Text::Splice(_, _) => {
                            quote_spanned! { text.span()=>
                                ::maple_core::generic_node::render::insert(
                                    ::std::clone::Clone::clone(&_el),
                                    ::maple_core::template_result::TemplateResult::new_lazy(move ||
                                        ::maple_core::render::IntoTemplate::create(&#text)
                                    ),
                                    None, None,
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

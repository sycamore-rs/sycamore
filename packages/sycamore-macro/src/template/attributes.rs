use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Paren;
use syn::{parenthesized, Expr, ExprLit, Ident, Lit, Result, Token};

#[derive(PartialEq, Eq)]
pub enum AttributeType {
    /// Syntax: `<name>`. `name` cannot be `dangerously_set_inner_html`.
    DomAttribute { name: AttributeName },
    /// Syntax: `dangerously_set_inner_html`.
    DangerouslySetInnerHtml,
    /// Syntax: `on:<event>`.
    Event { event: String },
    /// Syntax: `bind:<prop>`.
    Bind { prop: String },
    /// Syntax: `ref`.
    Ref,
}

impl Parse for AttributeType {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: AttributeName = input.parse()?;
        let ident_str = ident.to_string();

        if ident_str == "ref" {
            Ok(Self::Ref)
        } else if ident_str == "dangerously_set_inner_html" {
            Ok(Self::DangerouslySetInnerHtml)
        } else if input.peek(Token![:]) {
            let _colon: Token![:] = input.parse()?;
            match ident_str.as_str() {
                "on" => {
                    let event = input.call(Ident::parse_any)?;
                    Ok(Self::Event {
                        event: event.to_string(),
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
                    format!("unknown directive `{}`", ident_str),
                )),
            }
        } else {
            Ok(Self::DomAttribute { name: ident })
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

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        let expr_span = expr.span();

        let is_dynamic = !matches!(
            expr,
            Expr::Lit(ExprLit {
                lit: Lit::Str(_),
                ..
            })
        );

        match &self.ty {
            AttributeType::DomAttribute { name } => {
                let name = name.to_string();
                // Use `set_class_name` instead of `set_attribute` for better performance.
                let is_class = name == "class";
                let quoted_text = if let Expr::Lit(ExprLit {
                    lit: Lit::Str(text),
                    ..
                }) = expr
                {
                    quote_spanned! { text.span()=>
                        if ::std::cfg!(target_arch = "wasm32") {
                            ::sycamore::rt::intern(#text)
                        } else {
                            #text
                        }
                    }
                } else {
                    quote! {
                        &::std::string::ToString::to_string(&#expr)
                    }
                };
                let quoted_set_attribute = if is_class {
                    quote! {
                        ::sycamore::generic_node::GenericNode::set_class_name(&__el, #quoted_text);
                    }
                } else {
                    quote! {
                        ::sycamore::generic_node::GenericNode::set_attribute(
                            &__el,
                            #name,
                            #quoted_text,
                        );
                    }
                };

                if is_dynamic {
                    tokens.extend(quote_spanned! { expr_span=>
                        ::sycamore::reactive::create_effect({
                            let __el = ::std::clone::Clone::clone(&__el);
                            move || {
                                #quoted_set_attribute
                            }
                        });
                    });
                } else {
                    // Since this is static text, intern it as it will likely be constructed many
                    // times.
                    tokens.extend(quote_spanned! { expr_span=>
                        #quoted_set_attribute
                    });
                };
            }
            AttributeType::DangerouslySetInnerHtml => {
                if is_dynamic {
                    tokens.extend(quote_spanned! { expr_span=>
                        ::sycamore::reactive::create_effect({
                            let __el = ::std::clone::Clone::clone(&__el);
                            move || {
                                ::sycamore::generic_node::GenericNode::dangerously_set_inner_html(
                                    &__el,
                                    #expr,
                                );
                            }
                        });
                    });
                } else {
                    tokens.extend(quote_spanned! { expr_span=>
                        ::sycamore::generic_node::GenericNode::dangerously_set_inner_html(
                            &__el,
                            #expr,
                        );
                    });
                };
            }
            AttributeType::Event { event } => {
                // TODO: Should events be reactive?
                tokens.extend(quote_spanned! { expr_span=>
                    ::sycamore::generic_node::GenericNode::event(
                        &__el,
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
                        quote! { ::sycamore::rt::JsValue::from_bool(*signal.get()) }
                    }
                    JsPropertyType::String => {
                        quote! {
                            ::sycamore::rt::JsValue::from_str(
                                &::std::string::ToString::to_string(&signal.get())
                            )
                        }
                    }
                };

                let event_target_prop = quote! {
                    ::sycamore::rt::Reflect::get(
                        &event.target().unwrap(),
                        &::std::convert::Into::<::sycamore::rt::JsValue>::into(#prop)
                    ).unwrap()
                };

                let convert_from_jsvalue_fn = match property_ty {
                    JsPropertyType::Bool => quote! {
                        ::sycamore::rt::JsValue::as_bool(&#event_target_prop).unwrap()
                    },
                    JsPropertyType::String => quote! {
                        ::sycamore::rt::JsValue::as_string(&#event_target_prop).unwrap()
                    },
                };

                tokens.extend(quote_spanned! { expr_span=> {
                    let signal: ::sycamore::reactive::Signal<#value_ty> = #expr;

                    ::sycamore::reactive::create_effect({
                        let signal = ::std::clone::Clone::clone(&signal);
                        let __el = ::std::clone::Clone::clone(&__el);
                        move || {
                            ::sycamore::generic_node::GenericNode::set_property(
                                &__el,
                                #prop,
                                &#convert_into_jsvalue_fn,
                            );
                        }
                    });

                    ::sycamore::generic_node::GenericNode::event(
                        &__el,
                        #event_name,
                        ::std::boxed::Box::new(move |event: ::sycamore::rt::Event| {
                            signal.set(#convert_from_jsvalue_fn);
                        }),
                    )
                }});
            }
            AttributeType::Ref => {
                tokens.extend(quote_spanned! { expr_span=>{
                    ::sycamore::noderef::NodeRef::set(
                        &#expr,
                        ::std::clone::Clone::clone(&__el),
                    );
                }});
            }
        }
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

/// Represents a html element tag (e.g. `div`, `custom-element` etc...).
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

        write!(f, "{}", tag.to_string())?;
        for (_, ident) in extended {
            write!(f, "-{}", ident)?;
        }

        Ok(())
    }
}

impl PartialEq for AttributeName {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
impl Eq for AttributeName {}

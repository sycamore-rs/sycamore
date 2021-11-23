use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{token, Expr, Ident, Token};

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

        let tag = tag_name.to_string();
        let mut quoted = quote! {
            let __el = ::sycamore::generic_node::GenericNode::element(#tag);
        };

        let mut has_dangerously_set_inner_html = false;
        if let Some(attributes) = attributes {
            for attribute in &attributes.attributes {
                attribute.to_tokens(&mut quoted);
                if attribute.ty == AttributeType::DangerouslySetInnerHtml {
                    has_dangerously_set_inner_html = true;
                }
            }
        }

        if let Some(children) = children {
            if has_dangerously_set_inner_html && !children.body.is_empty() {
                quoted.extend(quote_spanned! { children.body[0].span()=>
                    compile_error!("children and inner html cannot be both set");
                });
            }

            let multi = children.body.len() != 1;
            let mut children = children.body.iter().peekable();
            while let Some(child) = children.next() {
                // Child is dynamic if the child is a component or a splice that is not a simple
                // path. Example:
                // view! { MyComponent() } // is_dynamic = true
                // view! { (state.get()) } // is_dynamic = true
                // view! { (state) } // is_dynamic = false
                quoted.extend(match child {
                    HtmlTree::Element(element) => quote_spanned! { element.span()=>
                        ::sycamore::generic_node::GenericNode::append_child(&__el, &#element);
                    },
                    HtmlTree::Text(text) => {
                        let intern = quote_spanned! { text.span()=>
                            // Since this is static text, intern it as it will likely be constructed many times.
                            if ::std::cfg!(target_arch = "wasm32") {
                                ::sycamore::rt::intern(#text);
                            }
                        };
                        if multi {
                            quote_spanned! { text.span()=>
                                #intern
                                ::sycamore::generic_node::GenericNode::append_child(
                                    &__el,
                                    &::sycamore::generic_node::GenericNode::text_node(#text),
                                );
                            }
                        } else {
                            // Only one child, directly set innerText instead of creating a text node.
                            quote_spanned! { text.span()=>
                                #intern
                                ::sycamore::generic_node::GenericNode::update_inner_text(&__el, #text);
                            }
                        }
                    },
                    HtmlTree::Splice(splice @ Splice {
                        expr: Expr::Lit(_) | Expr::Path(_), ..
                    }) => {
                        quote_spanned! { splice.span()=>
                            ::sycamore::utils::render::insert(
                                &__el,
                                ::sycamore::view::IntoView::create(&#splice),
                                None, None, #multi
                            );
                        }
                    },
                    // Child is dynamic.
                    HtmlTree::Component(_)
                    | HtmlTree::Splice(_) => {
                        let quote_marker =
                        if let Some(HtmlTree::Element(element)) =
                            children.next_if(|x| matches!(x, HtmlTree::Element(_)))
                        {
                            quote_spanned! { element.span()=>
                                let __marker = #element;
                                ::sycamore::generic_node::GenericNode::append_child(&__el, &__marker);
                                let __marker = ::std::option::Option::Some(&__marker);
                            }
                        } else if let Some(HtmlTree::Text(text)) =
                            children.next_if(|x| matches!(x, HtmlTree::Text(_)))
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
                        let initial = if cfg!(feature = "hydrate") {
                            quote! {
                                if ::std::any::Any::type_id(&__el) == ::std::any::TypeId::of::<::sycamore::HydrateNode>() {
                                    let __el = ::std::any::Any::downcast_ref::<::sycamore::HydrateNode>(&__el).unwrap();
                                    let __initial = ::sycamore::utils::hydrate::web::get_next_marker(&__el.inner_element());
                                    // Do not drop the HydrateNode because it will be cast into a GenericNode.
                                    let __initial = ::std::mem::ManuallyDrop::new(__initial);
                                    // SAFETY: This is safe because we already checked that the type is HydrateNode.
                                    // __initial is wrapped inside ManuallyDrop to prevent double drop.
                                    unsafe { ::std::ptr::read(&__initial as *const _ as *const _) }
                                } else {
                                    None
                                }
                            }
                        } else {
                            quote! {
                                None
                            }
                        };
                        match child {
                            HtmlTree::Component(component) => quote_spanned! { component.span()=>
                                #quote_marker
                                ::sycamore::utils::render::insert(
                                    &__el,
                                    #component,
                                    #initial, __marker, #multi
                                );
                            },
                            HtmlTree::Splice(splice) => quote_spanned! { splice.span()=>
                                #quote_marker
                                ::sycamore::utils::render::insert(
                                   &__el,
                                   ::sycamore::view::View::new_dyn(move ||
                                       ::sycamore::view::IntoView::create(&#splice)
                                   ),
                                   #initial, __marker, #multi
                               );
                            },
                            _ => unreachable!()
                        }
                    }
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

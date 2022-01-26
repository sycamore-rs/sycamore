//! Codegen for `view!` macro.
//!
//! Note: we are not using the `ToTokens` trait from `quote` because we need to keep track
//! of some internal state during the entire codegen.

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, Ident, Lit};

use crate::view::ir::*;

/// A struct for keeping track of the state when emitting Rust code.
pub struct Codegen {
    pub ctx: Ident,
}

impl Codegen {
    pub fn view_root(&self, view_root: &ViewRoot) -> TokenStream {
        match &view_root.0[..] {
            [] => quote! {
                ::sycamore::view::View::empty()
            },
            [node] => self.view_node(node),
            nodes => {
                let append_nodes: TokenStream = nodes
                    .iter()
                    .map(|node| {
                        let quoted = self.view_node(node);
                        quote! { children.push(#quoted); }
                    })
                    .collect();
                quote! {
                    ::sycamore::view::View::new_fragment({
                        let mut children = ::std::vec::Vec::new();
                        #append_nodes
                        children
                    })
                }
            }
        }
    }

    pub fn view_node(&self, view_node: &ViewNode) -> TokenStream {
        let ctx = &self.ctx;
        match view_node {
            ViewNode::Element(elem) => {
                let elem = self.element(elem);
                quote! {
                    ::sycamore::view::View::new_node(#elem)
                }
            }
            ViewNode::Component(comp) => self.component(comp),
            ViewNode::Text(Text { value }) => quote! {
                ::sycamore::view::View::new_node(::sycamore::generic_node::GenericNode::text_node(#value))
            },
            ViewNode::Dyn(Dyn { value }) => {
                quote! {
                    ::sycamore::view::View::new_dyn(#ctx, move ||
                        ::sycamore::view::IntoView::create(&(#value))
                    )
                }
            }
        }
    }

    pub fn element(&self, elem: &Element) -> TokenStream {
        let ctx = &self.ctx;
        let Element {
            tag,
            attrs,
            children,
        } = elem;

        let tag = match tag {
            ElementTag::Builtin(id) => id.to_string(),
            ElementTag::Custom(s) => s.clone(),
        };

        let quote_attrs: TokenStream = attrs.iter().map(|attr| self.attribute(attr)).collect();

        let quote_children = {
            let multi = children.len() >= 2;
            let mut children = children.iter().peekable();
            let mut quoted = TokenStream::new();
            while let Some(child) = children.next() {
                let is_dyn = child.is_dynamic();
                if is_dyn {
                    let codegen_ssr_markers = cfg!(feature = "ssr") && multi;
                    let mut marker_is_some = true;
                    let marker = if let Some(ViewNode::Element(elem)) =
                        children.next_if(|x| matches!(x, ViewNode::Element(_)))
                    {
                        let elem = self.element(elem);
                        quote! {
                            let __marker = #elem;
                            ::sycamore::generic_node::GenericNode::append_child(&__el, &__marker);
                            let __marker = ::std::option::Option::Some(&__marker);
                        }
                    } else if let Some(ViewNode::Text(Text { value })) =
                        children.next_if(|x| matches!(x, ViewNode::Text(_)))
                    {
                        quote! {
                            let __marker = ::sycamore::generic_node::GenericNode::text_node(#value);
                            ::sycamore::generic_node::GenericNode::append_child(&__el, &__marker);
                            let __marker = ::std::option::Option::Some(&__marker);
                        }
                    } else if children.peek().is_none() {
                        marker_is_some = false;
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
                    let marker_or_none = marker_is_some.then(|| marker.clone()).unwrap_or_default();

                    // If __el is a HydrateNode, use get_next_marker as initial node value.
                    let initial = if cfg!(feature = "experimental-hydrate") {
                        quote! {
                            if let ::std::option::Option::Some(__el)
                                = <dyn ::std::any::Any>::downcast_ref::<::sycamore::generic_node::HydrateNode>(&__el) {
                                let __initial = ::sycamore::utils::hydrate::web::get_next_marker(&__el.inner_element());
                                // Do not drop the HydrateNode because it will be cast into a GenericNode.
                                let __initial = ::std::mem::ManuallyDrop::new(__initial);
                                // SAFETY: This is safe because we already checked that the type is HydrateNode.
                                // __initial is wrapped inside ManuallyDrop to prevent double drop.
                                unsafe { ::std::ptr::read(&__initial as *const _ as *const _) }
                            } else { ::std::option::Option::None }
                        }
                    } else {
                        quote! { ::std::option::Option::None }
                    };
                    let ssr_markers = quote! {
                        ::sycamore::generic_node::GenericNode::append_child(
                            &__el,
                            &::sycamore::generic_node::GenericNode::marker_with_text("#"),
                        );
                        let __end_marker = ::sycamore::generic_node::GenericNode::marker_with_text("/");
                        ::sycamore::generic_node::GenericNode::append_child(&__el, &__end_marker);
                    };

                    quoted.extend(match child {
                        ViewNode::Component(comp) => {
                            let comp = self.component(comp);
                            let quoted = quote! {
                                #marker
                                ::sycamore::utils::render::insert(#ctx, &__el, #comp, #initial, __marker, #multi);
                            };
                            codegen_ssr_markers.then(|| quote! {
                                if ::std::any::Any::type_id(&__el) == ::std::any::TypeId::of::<::sycamore::generic_node::SsrNode>() {
                                    #ssr_markers
                                    ::sycamore::utils::render::insert(#ctx, &__el, #comp, #initial, Some(&__end_marker), #multi);
                                    #marker_or_none
                                } else { #quoted }
                            }).unwrap_or(quoted)
                        }
                        ViewNode::Dyn(Dyn { value}) => {
                            let quoted = quote! {
                                #marker
                                ::sycamore::utils::render::insert(#ctx, &__el,
                                    ::sycamore::view::View::new_dyn(#ctx, move ||
                                        ::sycamore::view::IntoView::create(&(#value))
                                    ),
                                    #initial, __marker, #multi
                                );
                            };
                            codegen_ssr_markers.then(|| quote!{
                                if ::std::any::Any::type_id(&__el) == ::std::any::TypeId::of::<::sycamore::generic_node::SsrNode>() {
                                    #ssr_markers
                                    ::sycamore::utils::render::insert(#ctx, &__el,
                                        ::sycamore::view::View::new_dyn(#ctx, move ||
                                            ::sycamore::view::IntoView::create(&(#value))
                                        ),
                                        #initial, Some(&__end_marker), #multi
                                    );
                                    #marker_or_none
                                } else { #quoted }
                            }).unwrap_or(quoted)
                        },
                        _ => unreachable!("only component and dyn node can be dynamic"),
                    });

                    // Do not perform non dynamic codegen.
                    continue;
                }
                match child {
                    ViewNode::Element(elem) => quoted.extend({
                        let elem = self.element(elem);
                        quote! {
                            ::sycamore::generic_node::GenericNode::append_child(&__el, &#elem);
                        }
                    }),
                    ViewNode::Component(_) => unreachable!("component is always dynamic"),
                    ViewNode::Text(Text { value }) => {
                        let intern = quote! {
                            // Since this is static text, intern it as it will likely be constructed many times.
                            #[cfg(target_arch = "wasm32")]
                            ::sycamore::rt::intern(#value);
                        };
                        quoted.extend(match multi {
                            true => quote! {
                                #intern
                                ::sycamore::generic_node::GenericNode::append_child(
                                    &__el,
                                    &::sycamore::generic_node::GenericNode::text_node(#value),
                                );
                            },
                            // Only one child, directly set innerText instead of creating a text node.
                            false => quote! {
                                #intern
                                ::sycamore::generic_node::GenericNode::update_inner_text(&__el, #value);
                            },
                        });
                    }
                    ViewNode::Dyn(Dyn { value }) => quoted.extend(quote! {
                        ::sycamore::utils::render::insert(#ctx, &__el,
                            ::sycamore::view::IntoView::create(&(#value)),
                            None, None, #multi
                        );
                    }),
                }
            }
            quoted
        };

        quote! {{
            let __el = ::sycamore::generic_node::GenericNode::element(#tag);
            #quote_attrs
            #quote_children
            __el
        }}
    }

    pub fn attribute(&self, attr: &Attribute) -> TokenStream {
        let ctx = &self.ctx;
        let mut tokens = TokenStream::new();
        let expr = &attr.value;

        let is_dynamic = !matches!(
            expr,
            Expr::Lit(ExprLit {
                lit: Lit::Str(_),
                ..
            })
        );

        match &attr.ty {
            AttributeType::Str { name } => {
                let name = name.to_string();
                // Use `set_class_name` instead of `set_attribute` for better performance.
                let is_class = name == "class";
                let quoted_text = if let Expr::Lit(ExprLit {
                    lit: Lit::Str(text),
                    ..
                }) = expr
                {
                    // Since this is static text, intern it as it will likely be constructed many
                    // times.
                    quote! {
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
                    tokens.extend(quote! {
                        ::sycamore::reactive::Scope::create_effect(#ctx, {
                            let __el = ::std::clone::Clone::clone(&__el);
                            move || { #quoted_set_attribute }
                        });
                    });
                } else {
                    tokens.extend(quote! { #quoted_set_attribute });
                };
            }
            AttributeType::Bool { name } => {
                let name = name.to_string();
                let quoted_set_attribute = quote! {
                    if #expr {
                        ::sycamore::generic_node::GenericNode::set_attribute(&__el, #name, "");
                    } else {
                        ::sycamore::generic_node::GenericNode::remove_attribute(&__el, #name);
                    }
                };

                if is_dynamic {
                    tokens.extend(quote! {
                        ::sycamore::reactive::Scope::create_effect(#ctx, {
                            let __el = ::std::clone::Clone::clone(&__el);
                            move || {
                                #quoted_set_attribute
                            }
                        });
                    });
                } else {
                    tokens.extend(quote! {
                        #quoted_set_attribute
                    });
                };
            }
            AttributeType::DangerouslySetInnerHtml => {
                if is_dynamic {
                    tokens.extend(quote! {
                        ::sycamore::reactive::Scope::create_effect(#ctx, {
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
                    tokens.extend(quote! {
                        ::sycamore::generic_node::GenericNode::dangerously_set_inner_html(
                            &__el,
                            #expr,
                        );
                    });
                };
            }
            AttributeType::Event { event } => {
                tokens.extend(quote! {
                    ::sycamore::generic_node::GenericNode::event(
                        &__el,
                        #ctx,
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
                                &format!("property `{}` is not supported with `bind:`", prop),
                            )
                            .to_compile_error(),
                        );
                        return tokens;
                    }
                };

                // let value_ty = match property_ty {
                //     JsPropertyType::Bool => quote! { ::std::primitive::bool },
                //     JsPropertyType::String => quote! { ::std::string::String },
                // };

                let convert_into_jsvalue_fn = match property_ty {
                    JsPropertyType::Bool => {
                        quote! { ::sycamore::rt::JsValue::from_bool(*#expr.get()) }
                    }
                    JsPropertyType::String => {
                        quote! {
                            ::sycamore::rt::JsValue::from_str(
                                &::std::string::ToString::to_string(&#expr.get())
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

                tokens.extend(quote! {
                    #[cfg(target_arch = "wasm32")]
                    ::sycamore::reactive::Scope::create_effect(#ctx, {
                        let __el = ::std::clone::Clone::clone(&__el);
                        move ||::sycamore::generic_node::GenericNode::set_property(
                            &__el,
                            #prop,
                            &#convert_into_jsvalue_fn,
                        )
                    });
                    ::sycamore::generic_node::GenericNode::event(&__el, #ctx, #event_name,
                        ::std::boxed::Box::new(|event: ::sycamore::rt::Event| {
                            #expr.set(#convert_from_jsvalue_fn);
                        }),
                    );
                });
            }
            AttributeType::Ref => {
                tokens.extend(quote! {{
                    ::sycamore::noderef::NodeRef::set(&#expr, ::std::clone::Clone::clone(&__el));
                }});
            }
        }
        tokens
    }

    pub fn component(&self, comp: &Component) -> TokenStream {
        let ctx = &self.ctx;
        match comp {
            Component::FnLike(comp) => {
                let FnLikeComponent { ident, args } = comp;
                if args.empty_or_trailing() {
                    quote! { ::sycamore::component::component_scope(move || #ident(#ctx, ())) }
                } else {
                    quote! { ::sycamore::component::component_scope(move || #ident(#ctx, #args)) }
                }
            }
            Component::ElementLike(comp) => {
                let ElementLikeComponent { ident, props } = comp;
                let mut props_quoted = quote! {
                    ::sycamore::component::element_like_component_builder(__component)
                };
                for (field, expr) in props {
                    props_quoted.extend(quote! { .#field(#expr) });
                }
                props_quoted.extend(quote! { .build() });

                quote! {{
                    let __component = &#ident; // We do this to make sure the compiler can infer the value for `<G>`.
                    ::sycamore::component::component_scope(move || __component(#ctx, #props_quoted))
                }}
            }
        }
    }
}

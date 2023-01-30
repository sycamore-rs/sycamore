//! Codegen for `view!` macro.
//!
//! Note: we are not using the `ToTokens` trait from `quote` because we need to keep track
//! of some internal state during the entire codegen.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Path};
use syn::{Expr, ExprLit, Ident, Lit, LitBool};

use crate::view::ir::*;

/// A struct for keeping track of the state when emitting Rust code.
pub struct Codegen {
    pub elements_mod_path: syn::Path,
    pub cx: Ident,
}

impl Codegen {
    pub fn view_root(&self, view_root: &ViewRoot) -> TokenStream {
        match &view_root.0[..] {
            [] => quote! {
                ::sycamore::view::View::empty()
            },
            [node] => self.view_node(node),
            nodes => {
                let nodes = nodes.iter().map(|node| self.view_node(node));
                quote! {
                    ::sycamore::view::View::new_fragment({
                        ::std::vec![
                            #(#nodes),*
                        ]
                    })
                }
            }
        }
    }

    /// Generate a `View` from a `ViewNode`. If the root is an element, create a `Template`.
    pub fn view_node(&self, view_node: &ViewNode) -> TokenStream {
        let cx = &self.cx;
        match view_node {
            ViewNode::Element(_) => {
                let template_id = rand::random::<u32>();

                let mut codegen =
                    CodegenTemplate::new(self.elements_mod_path.clone(), self.cx.clone());

                let shape = codegen.node(view_node);
                let dyn_values = codegen.dyn_values;
                let flagged_nodes_quoted = codegen.flagged_nodes_quoted;
                quote! {{
                    use ::sycamore::generic_node::SycamoreElement as _;

                    static __TEMPLATE: ::sycamore::generic_node::Template = ::sycamore::generic_node::Template {
                        id: ::sycamore::generic_node::TemplateId(#template_id),
                        shape: #shape,
                    };

                    let __dyn_values = ::std::vec![#(#dyn_values),*];
                    let __result = ::sycamore::generic_node::__instantiate_template(&__TEMPLATE);
                    ::sycamore::generic_node::__apply_dyn_values_to_template(#cx, &__result.dyn_markers, __dyn_values);
                    let __flagged = __result.flagged_nodes;
                    #flagged_nodes_quoted

                    ::sycamore::view::View::new_node(__result.root)
                }}
            }
            ViewNode::Component(component) => {
                impl_component(&self.elements_mod_path, &self.cx, component)
            }
            ViewNode::Text(Text { value }) => quote! {
                ::sycamore::view::View::new_node(::sycamore::generic_node::GenericNode::text_node(::std::borrow::Cow::Borrowed(#value)))
            },
            ViewNode::Dyn(dyn_node @ Dyn { value }) => {
                let cx = &self.cx;
                let needs_cx = dyn_node.needs_cx(&cx.to_string());

                match needs_cx {
                    true => quote! {
                        ::sycamore::view::View::new_dyn_scoped(#cx, move |#cx| ::sycamore::view::ToView::to_view(&(#value)))
                    },
                    false => quote! {
                        ::sycamore::view::View::new_dyn(#cx, move || ::sycamore::view::ToView::to_view(&(#value)))
                    },
                }
            }
        }
    }
}

/// Codegen a `TemplateShape`.
struct CodegenTemplate {
    elements_mod_path: syn::Path,
    cx: Ident,
    flag_counter: usize,
    flagged_nodes_quoted: TokenStream,
    dyn_values: Vec<TokenStream>,
}

impl CodegenTemplate {
    fn new(elements_mod_path: syn::Path, cx: Ident) -> Self {
        Self {
            elements_mod_path,
            cx,
            flag_counter: 0,
            flagged_nodes_quoted: TokenStream::new(),
            dyn_values: Vec::new(),
        }
    }

    fn node(&mut self, node: &ViewNode) -> TokenStream {
        match node {
            ViewNode::Element(element) => self.element(element),
            ViewNode::Component(component) => self.component(component),
            ViewNode::Text(text) => self.text(text),
            ViewNode::Dyn(dyn_node) => self.dyn_marker(dyn_node),
        }
    }

    fn element(&mut self, element: &Element) -> TokenStream {
        let elements_mod_path = self.elements_mod_path.clone();

        let Element {
            tag,
            attrs,
            children,
        } = element;

        let attrs = attrs
            .iter()
            .map(|attr| self.attribute(attr))
            .collect::<Vec<_>>();
        let flag = attrs.iter().any(|(_, flag)| *flag);
        let attrs = attrs.into_iter().filter_map(|(attr, _)| attr);

        if flag {
            self.flag_counter += 1;
        }
        // We run codegen for children after attrs to make sure that we have the correct flag
        // counter.
        let children = children
            .iter()
            .map(|child| self.node(child))
            .collect::<Vec<_>>();

        match tag {
            ElementTag::Builtin(tag) => quote! {{
                type __tag = #elements_mod_path::#tag;
                ::sycamore::generic_node::TemplateShape::Element {
                    tag: __tag::TAG_NAME,
                    ns: __tag::NAMESPACE,
                    children: &[#(#children),*],
                    attributes: &[#(#attrs),*],
                    flag: #flag,
                }
            }},
            ElementTag::Custom(tag) => quote! {
                ::sycamore::generic_node::TemplateShape::Element {
                    tag: #tag,
                    ns: ::std::option::Option::None,
                    children: &[#(#children),*],
                    attributes: &[#(#attrs),*],
                    flag: #flag,
                }
            },
        }
    }

    fn attribute(&mut self, attr: &Attribute) -> (Option<TokenStream>, bool) {
        let cx = &self.cx;
        let elements_mod_path = &self.elements_mod_path;
        let flag_counter = self.flag_counter;

        let expr = &attr.value;

        let is_dynamic = !matches!(expr, Expr::Lit(ExprLit { .. }));

        match &attr.ty {
            AttributeType::Str { name } => {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(text),
                    ..
                }) = expr
                {
                    (
                        Some(quote! { (#name, ::std::borrow::Cow::Borrowed(#text)) }),
                        false,
                    )
                } else {
                    let text = quote! { ::std::borrow::Cow::Owned(::std::string::ToString::to_string(&#expr)) };
                    if is_dynamic {
                        self.flagged_nodes_quoted.extend(quote! {
                            let __el = ::std::clone::Clone::clone(&__flagged[#flag_counter]);
                            ::sycamore::reactive::create_effect(#cx, move || {
                                ::sycamore::generic_node::GenericNode::set_attribute(&__el, ::std::borrow::Cow::Borrowed(#name), #text);
                            });
                        });
                    } else {
                        self.flagged_nodes_quoted.extend(quote! {
                            ::sycamore::generic_node::GenericNode::set_attribute(&__flagged[#flag_counter], ::std::borrow::Cow::Borrowed(#name), #text);
                        });
                    }
                    (None, true)
                }
            }
            AttributeType::Bool { name } => {
                if is_dynamic {
                    let quoted_set_attribute = quote! {
                        if #expr {
                            ::sycamore::generic_node::GenericNode::set_attribute(&__el, ::std::borrow::Cow::Borrowed(#name), ::std::borrow::Cow::Borrowed(""));
                        } else {
                            ::sycamore::generic_node::GenericNode::remove_attribute(&__el, ::std::borrow::Cow::Borrowed(#name));
                        }
                    };
                    self.flagged_nodes_quoted.extend(quote! {
                        let __el = ::std::clone::Clone::clone(&__flagged[#flag_counter]);
                        ::sycamore::reactive::create_effect(#cx, move || { #quoted_set_attribute });
                    });
                    (None, true)
                } else if let Expr::Lit(ExprLit {
                    lit: Lit::Bool(LitBool { value, .. }),
                    ..
                }) = expr
                {
                    let stringified = match value {
                        true => "true",
                        false => "false",
                    };
                    (
                        Some(quote! { (#name, ::std::borrow::Cow::Borrowed(#stringified)) }),
                        false,
                    )
                } else {
                    // Wrong type. Produce a type error.
                    (
                        Some(
                            quote! { (#name, { let _e: ::std::primitive::bool = #expr; ::std::borrow::Cow::Borrowed("") }) },
                        ),
                        false,
                    )
                }
            }
            AttributeType::DangerouslySetInnerHtml => {
                if is_dynamic {
                    self.flagged_nodes_quoted.extend(quote! {
                        let __el = ::std::clone::Clone::clone(&__flagged[#flag_counter]);
                        ::sycamore::reactive::create_effect(#cx, move || {
                            ::sycamore::generic_node::GenericNode::dangerously_set_inner_html(
                                &__el,
                                <_ as ::std::convert::Into<_>>::into(#expr),
                            );
                        });
                    });
                    (None, true)
                } else {
                    self.flagged_nodes_quoted.extend(quote! {
                        ::sycamore::generic_node::GenericNode::dangerously_set_inner_html(
                            &__flagged[#flag_counter],
                            <_ as ::std::convert::Into<_>>::into(#expr),
                        );
                    });
                    (None, true)
                }
            }
            AttributeType::Event { event } => {
                self.flagged_nodes_quoted.extend(quote! {
                    ::sycamore::generic_node::GenericNode::event(
                        &__flagged[#flag_counter],
                        #cx,
                        #elements_mod_path::ev::#event,
                        #expr,
                    );
                });
                (None, true)
            }
            AttributeType::Property { prop } => {
                let value = quote! { ::std::convert::Into::<::sycamore::rt::JsValue>::into(#expr) };

                if is_dynamic {
                    self.flagged_nodes_quoted.extend(quote! {
                        let __el = ::std::clone::Clone::clone(&__flagged[#flag_counter]);
                        ::sycamore::reactive::create_effect(#cx, move ||
                            ::sycamore::generic_node::GenericNode::set_property(&__el, #prop, &#value)
                        );
                    });
                } else {
                    self.flagged_nodes_quoted.extend(quote! {
                        ::sycamore::generic_node::GenericNode::set_property(&__flagged[#flag_counter], #prop, &#value);
                    });
                }

                (None, true)
            }
            AttributeType::Bind { prop } => {
                #[derive(Clone, Copy)]
                enum JsPropertyType {
                    Bool,
                    String,
                    Number,
                }

                let (event_name, property_ty) = match prop.as_str() {
                    "value" => (quote! { input }, JsPropertyType::String),
                    "valueAsNumber" => (quote! { input }, JsPropertyType::Number),
                    "checked" => (quote! { change }, JsPropertyType::Bool),
                    _ => {
                        self.flagged_nodes_quoted.extend(
                            syn::Error::new(
                                prop.span(),
                                format!("property `{}` is not supported with `bind:`", prop),
                            )
                            .to_compile_error(),
                        );
                        return (None, false);
                    }
                };

                let convert_into_jsvalue_fn = match property_ty {
                    JsPropertyType::Bool => quote! {
                        ::sycamore::rt::JsValue::from_bool(
                            *::sycamore::reactive::ReadSignal::get(&__expr)
                        )
                    },
                    JsPropertyType::Number => quote! {
                        ::sycamore::rt::JsValue::from_f64(
                            *::sycamore::reactive::ReadSignal::get(&__expr)
                        )
                    },
                    JsPropertyType::String => quote! {
                        ::sycamore::rt::JsValue::from_str(
                            &::std::string::ToString::to_string(
                                &::sycamore::reactive::ReadSignal::get(&__expr),
                            )
                        )
                    },
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
                    JsPropertyType::Number => quote! {
                        ::sycamore::rt::JsValue::as_f64(&#event_target_prop).unwrap()
                    },
                    JsPropertyType::String => quote! {
                        ::sycamore::rt::JsValue::as_string(&#event_target_prop).unwrap()
                    },
                };

                self.flagged_nodes_quoted.extend(quote! {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let __el = ::std::clone::Clone::clone(&__flagged[#flag_counter]);
                        ::sycamore::reactive::create_effect(#cx, {
                            let __expr = ::std::clone::Clone::clone(&#expr);
                            move ||::sycamore::generic_node::GenericNode::set_property(
                                &__el,
                                #prop,
                                &#convert_into_jsvalue_fn,
                            )
                        });
                    }
                    ::sycamore::generic_node::GenericNode::event(
                        &__flagged[#flag_counter], #cx, #elements_mod_path::ev::#event_name,
                        {
                            let __expr = ::std::clone::Clone::clone(&#expr);
                            ::std::boxed::Box::new(move |event: ::sycamore::rt::Event| {
                                ::sycamore::reactive::Signal::set(&__expr, #convert_from_jsvalue_fn);
                            })
                        },
                    );
                });

                (None, true)
            }
            AttributeType::Ref => {
                self.flagged_nodes_quoted.extend(quote! {
                    ::sycamore::noderef::NodeRef::set(&#expr, ::std::clone::Clone::clone(&__flagged[#flag_counter]));
                });
                (None, true)
            }
            AttributeType::Spread => {
                self.flagged_nodes_quoted.extend(quote! {
                    let __el = ::std::clone::Clone::clone(&__flagged[#flag_counter]);
                    for (name, value) in #expr.drain() {
                        let __el = ::std::clone::Clone::clone(&__flagged[#flag_counter]);
                        ::sycamore::utils::apply_attribute(#cx, __el, ::std::clone::Clone::clone(&name), value);
                    }
                });
                (None, true)
            }
        }
    }

    fn component(&mut self, component: &Component) -> TokenStream {
        // Add a DynMarker and set the component as a dyn value.
        self.dyn_values
            .push(impl_component(&self.elements_mod_path, &self.cx, component));
        quote! {
            ::sycamore::generic_node::TemplateShape::DynMarker
        }
    }

    fn text(&mut self, Text { value }: &Text) -> TokenStream {
        quote! {
            ::sycamore::generic_node::TemplateShape::Text(#value)
        }
    }

    fn dyn_marker(&mut self, dyn_node @ Dyn { value }: &Dyn) -> TokenStream {
        let cx = &self.cx;
        let needs_cx = dyn_node.needs_cx(&cx.to_string());

        let dyn_node = match needs_cx {
            true => quote! {
                ::sycamore::view::View::new_dyn_scoped(#cx, move |#cx| ::sycamore::view::ToView::to_view(&(#value)))
            },
            false => quote! {
                ::sycamore::view::View::new_dyn(#cx, move || ::sycamore::view::ToView::to_view(&(#value)))
            },
        };
        self.dyn_values.push(dyn_node);

        quote! {
            ::sycamore::generic_node::TemplateShape::DynMarker
        }
    }
}

fn impl_component(elements_mod_path: &syn::Path, cx: &Ident, component: &Component) -> TokenStream {
    let Component {
        ident,
        props,
        children,
        ..
    } = component;

    let prop_names = props
        .iter()
        .filter(|prop| prop.prefix.is_none())
        .map(|x| format_ident!("{}", &x.name));
    let prop_values = props
        .iter()
        .filter(|prop| prop.prefix.is_none())
        .map(|x| &x.value);

    let attributes = props
        .iter()
        .filter(|prop| prop.prefix.is_some())
        .map(|prop| (&prop.prefix, &prop.name, &prop.value));
    let attribute_entries_quoted = attributes
        .map(|(prefix, name, value)| {
            let prefix = prefix.as_ref().unwrap();
            let value = to_attribute_value(prefix, name, value, cx, elements_mod_path)?;
            let name_str = if prefix == "attr" {
                name.to_string()
            } else {
                format!("{prefix}:{name}")
            };
            Ok(quote! {
                attributes.insert(::std::borrow::Cow::Borrowed(#name_str), #value)
            })
        })
        .collect::<Result<Vec<_>, syn::Error>>()
        .map_err(|err| err.to_compile_error());
    let attributes_quoted = if let Ok(attributes) = attribute_entries_quoted {
        if !attributes.is_empty() {
            quote! {
                .attributes({
                    let mut attributes = ::std::collections::HashMap::default();
                    #(#attributes;)*
                    ::sycamore::component::Attributes::new(attributes)
                })
            }
        } else {
            quote!()
        }
    } else {
        quote!()
    };

    let children_quoted = children
        .as_ref()
        .filter(|children| !children.0.is_empty())
        .map(|children| {
            let codegen = Codegen {
                elements_mod_path: elements_mod_path.clone(),
                cx: cx.clone(),
            };
            let children = codegen.view_root(children);
            quote! {
                .children(
                    ::sycamore::component::Children::new(#cx, move |#cx| {
                        #[allow(unused_variables)]
                        let #cx: ::sycamore::reactive::BoundedScope = #cx;
                        #children
                    })
                )
            }
        })
        .unwrap_or_default();
    quote! {{
        let __component = &#ident; // We do this to make sure the compiler can infer the value for `<G>`.
        ::sycamore::component::component_scope(move || ::sycamore::component::Component::create(
            __component,
            #cx,
            ::sycamore::component::element_like_component_builder(__component)
                #(.#prop_names(#prop_values))*
                #children_quoted
                #attributes_quoted
                .build()
        ))
    }}
}

fn to_attribute_value(
    prefix: &Ident,
    name: &str,
    value: &Expr,
    cx: &Ident,
    elements_mod_path: &Path,
) -> Result<TokenStream, syn::Error> {
    match prefix.to_string().as_str() {
        "on" => {
            let event = format_ident!("{name}");

            Ok(quote!(::sycamore::component::AttributeValue::Event(
                #name,
                ::sycamore::utils::erase_handler::<#elements_mod_path::ev::#event, _, _>(#cx, #value)
            )))
        }
        "prop" => Ok(quote!(::sycamore::component::AttributeValue::Property(#name, #value))),
        "bind" => match name {
            "value" => {
                Ok(quote!(::sycamore::component::AttributeValue::BindString("value", #value)))
            }
            "valueAsNumber" => Ok(
                quote!(::sycamore::component::AttributeValue::BindNumber("valueAsNumber", #value)),
            ),
            "checked" => {
                Ok(quote!(::sycamore::component::AttributeValue::BindBool("checked", #value)))
            }
            _ => Err(syn::Error::new(
                name.span(),
                format!("property `{}` is not supported with `bind:`", name),
            )),
        },
        "attr" => {
            if name == "ref" {
                Ok(quote!(::sycamore::component::AttributeValue::Ref(#value)))
            } else if name == "dangerously_set_inner_html" {
                if matches!(value, Expr::Lit(_)) {
                    Ok(
                        quote!(::sycamore::component::AttributeValue::DangerouslySetInnerHtml(#value.to_string())),
                    )
                } else {
                    Ok(
                        quote!(::sycamore::component::AttributeValue::DynamicDangerouslySetInnerHtml(Box::new(#value))),
                    )
                }
            } else if is_bool_attr(name) {
                if matches!(value, Expr::Lit(_)) {
                    Ok(quote!(::sycamore::component::AttributeValue::Bool(#value)))
                } else {
                    Ok(quote!(::sycamore::component::AttributeValue::DynamicBool(
                        Box::new(move || #value)
                    )))
                }
            } else if matches!(value, Expr::Lit(_)) {
                Ok(quote!(::sycamore::component::AttributeValue::Str(#value)))
            } else {
                Ok(quote!(::sycamore::component::AttributeValue::DynamicStr(
                    Box::new(move || ::std::string::ToString::to_string(#value))
                )))
            }
        }
        _ => Err(syn::Error::new_spanned(
            name,
            format!("unknown directive `{}`", name),
        )),
    }
}

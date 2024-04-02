//! Codegen for `view!` macro.
//!
//! Implementation note: We are not using the `quote::ToTokens` trait because we need to pass
//! additional information to the codegen such as which mode (Client, Hydrate, SSR), etc...

use std::collections::HashSet;

use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use quote::quote;
use sycamore_view_parser::ir::{DynNode, Node, Prop, PropType, Root, TagIdent, TagNode, TextNode};
use syn::{Expr, Ident};

pub struct Codegen {
    // TODO: configure mode: Client, Hydrate, SSR
}

impl Codegen {
    pub fn root(&self, root: &Root) -> TokenStream {
        match &root.0[..] {
            [] => quote! {
                ::sycamore::rt::View::new()
            },
            [node] => self.node(node),
            nodes => {
                let nodes = nodes.iter().map(|node| self.node(node));
                quote! {
                    ::std::convert::Into::<::sycamore::rt::View>::into(::std::vec![#(#nodes),*])
                }
            }
        }
    }

    /// Generate a `View` from a `Node`.
    pub fn node(&self, node: &Node) -> TokenStream {
        match node {
            Node::Tag(tag) => {
                if is_component(&tag.ident) {
                    self.component(tag)
                } else {
                    self.element(tag)
                }
            }
            Node::Text(TextNode { value }) => quote! {
                ::std::convert::Into::<::sycamore::rt::View>::into(#value)
            },
            Node::Dyn(DynNode { value }) => {
                quote! {
                    ::std::convert::Into::<::sycamore::rt::View>::into(
                        move || ::std::convert::Into::<::sycamore::rt::View>::into(&(#value))
                    )
                }
            }
        }
    }

    pub fn element(&self, element: &TagNode) -> TokenStream {
        let TagNode {
            ident,
            props,
            children,
        } = element;

        let attributes = props.iter().map(|attr| self.attribute(attr));

        let children = children
            .0
            .iter()
            .map(|child| self.node(child))
            .collect::<Vec<_>>();

        match ident {
            TagIdent::Path(tag) => {
                assert!(tag.get_ident().is_some(), "elements must be an ident");
                quote! {
                    ::sycamore::rt::View::from(
                        ::sycamore::rt::tags::#tag().children(::std::vec![#(#children),*])#(#attributes)*
                    )
                }
            }
            TagIdent::Hyphenated(tag) => quote! {
                ::sycamore::rt::View::from(
                    ::sycamore::rt::custom_element(#tag).children(::std::vec![#(#children),*])#(#attributes)*
                )
            },
        }
    }

    pub fn attribute(&self, attr: &Prop) -> TokenStream {
        let value = &attr.value;
        let is_dynamic = !matches!(value, Expr::Lit(_) | Expr::Closure(_));
        let dyn_value = if is_dynamic {
            quote! { move || #value }
        } else {
            quote! { #value }
        };
        match &attr.ty {
            PropType::Plain { ident } => {
                quote! { .#ident(#dyn_value) }
            }
            PropType::PlainHyphenated { ident } => {
                quote! { .attr(#ident, #dyn_value) }
            }
            PropType::Directive { dir, ident } => match dir.to_string().as_str() {
                "on" => quote! { .on(::sycamore::rt::events::#ident, #value) },
                "prop" => {
                    let ident = ident.to_string();
                    quote! { .prop(#ident, #dyn_value) }
                }
                "bind" => {
                    #[derive(Clone, Copy)]
                    enum JsPropertyType {
                        Bool,
                        String,
                        Number,
                    }

                    // let span = ident.span();
                    // let ident = ident.to_string();
                    // let (event_name, property_ty) = match ident.as_str() {
                    //     "value" => (quote! { input }, JsPropertyType::String),
                    //     "valueAsNumber" => (quote! { input }, JsPropertyType::Number),
                    //     "checked" => (quote! { change }, JsPropertyType::Bool),
                    //     _ => {
                    //         self.flagged_nodes_quoted.extend(
                    //             syn::Error::new(
                    //                 span,
                    //                 format!("property `{}` is not supported with `bind:`",
                    // ident),             )
                    //             .to_compile_error(),
                    //         );
                    //         return (None, false);
                    //     }
                    // };
                    //
                    // let convert_into_jsvalue_fn = match property_ty {
                    //     JsPropertyType::Bool => quote! {
                    //         ::sycamore::rt::JsValue::from_bool(__expr.get())
                    //     },
                    //     JsPropertyType::Number => quote! {
                    //         ::sycamore::rt::JsValue::from_f64(__expr.get())
                    //     },
                    //     JsPropertyType::String => quote! {
                    //         __expr.with(|__expr|
                    //
                    // ::sycamore::rt::JsValue::from_str(&
                    // ::std::string::ToString::to_string(__expr))         )
                    //     },
                    // };
                    //
                    // let event_target_prop = quote! {
                    //     ::sycamore::rt::Reflect::get(
                    //         &event.target().unwrap(),
                    //         &::std::convert::Into::<::sycamore::rt::JsValue>::into(#ident)
                    //     ).unwrap()
                    // };
                    //
                    // let convert_from_jsvalue_fn = match property_ty {
                    //     JsPropertyType::Bool => quote! {
                    //         ::sycamore::rt::JsValue::as_bool(&#event_target_prop).unwrap()
                    //     },
                    //     JsPropertyType::Number => quote! {
                    //         ::sycamore::rt::JsValue::as_f64(&#event_target_prop).unwrap()
                    //     },
                    //     JsPropertyType::String => quote! {
                    //         ::sycamore::rt::JsValue::as_string(&#event_target_prop).unwrap()
                    //     },
                    // };
                    //
                    // self.flagged_nodes_quoted.extend(quote! {
                    //     #[cfg(target_arch = "wasm32")]
                    //     {
                    //         let __el = ::std::clone::Clone::clone(&__flagged[#flag_counter]);
                    //         ::sycamore::reactive::create_effect({
                    //             let __expr = ::std::clone::Clone::clone(&#value);
                    //             move ||::sycamore::generic_node::GenericNode::set_property(
                    //                 &__el,
                    //                 #ident,
                    //                 &#convert_into_jsvalue_fn,
                    //             )
                    //         });
                    //     }
                    //     ::sycamore::generic_node::GenericNode::event(
                    //         &__flagged[#flag_counter], ::sycamore::web::html::ev::#event_name,
                    //         {
                    //             let __expr = ::std::clone::Clone::clone(&#value);
                    //             ::std::boxed::Box::new(move |event: ::sycamore::rt::Event| {
                    //                 ::sycamore::reactive::Signal::set(__expr,
                    // #convert_from_jsvalue_fn);             })
                    //         },
                    //     );
                    // });
                    // (None, true)
                    todo!("bind directive")
                }
                _ => syn::Error::new(dir.span(), format!("unknown directive `{dir}`"))
                    .to_compile_error(),
            },
            PropType::Ref => todo!(),
            PropType::Spread => todo!(),
        }
    }

    pub fn component(
        &self,
        TagNode {
            ident,
            props,
            children,
        }: &TagNode,
    ) -> TokenStream {
        let ident = match ident {
            TagIdent::Path(path) => path,
            TagIdent::Hyphenated(_) => unreachable!("hyphenated tags are not components"),
        };

        let plain = props
            .iter()
            .filter_map(|prop| match &prop.ty {
                PropType::Plain { ident } => Some((ident, prop.value.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        let plain_names = plain.iter().map(|(ident, _)| ident);
        let plain_values = plain.iter().map(|(_, value)| value);

        let attributes = props.iter().filter_map(|prop| match &prop.ty {
            PropType::Directive { dir, ident } => Some((dir, ident, prop.value.clone())),
            _ => None,
        });
        let attribute_entries_quoted = attributes
            .map(|(dir, name, value)| {
                let value = to_attribute_value(dir, name, &value)?;
                let name_str = if dir == "attr" {
                    name.to_string()
                } else {
                    format!("{dir}:{name}")
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
                        ::sycamore::rt::Attributes::new(attributes)
                    })
                }
            } else {
                quote!()
            }
        } else {
            quote!()
        };

        let children_quoted = if children.0.is_empty() {
            quote! {}
        } else {
            let codegen = Codegen {};
            let children = codegen.root(children);
            quote! {
                .children(
                    ::sycamore::rt::Children::new(move || {
                        #children
                    })
                )
            }
        };
        quote! {{
            let __component = &#ident; // We do this to make sure the compiler can infer the value for `<G>`.
            ::sycamore::rt::component_scope(move || ::sycamore::rt::Component::create(
                __component,
                ::sycamore::rt::element_like_component_builder(__component)
                    #(.#plain_names(#plain_values))*
                    #children_quoted
                    #attributes_quoted
                    .build()
            ))
        }}
    }
}

fn to_attribute_value(dir: &Ident, ident: &Ident, value: &Expr) -> Result<TokenStream, syn::Error> {
    match dir.to_string().as_str() {
        "on" => {
            let event = ident.to_string();

            Ok(quote!(::sycamore::component::AttributeValue::Event(
                #event,
                ::sycamore::utils::erase_handler::<::sycamore::web::html::ev::#ident, _, _>(#value)
            )))
        }
        "prop" => Ok(quote!(::sycamore::component::AttributeValue::Property(#ident, #value))),
        "bind" => match ident.to_string().as_str() {
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
                ident.span(),
                format!("property `{}` is not supported with `bind:`", ident),
            )),
        },
        "attr" => {
            if ident == "ref" {
                Ok(quote!(::sycamore::component::AttributeValue::Ref(#value)))
            } else if ident == "dangerously_set_inner_html" {
                if matches!(value, Expr::Lit(_)) {
                    Ok(
                        quote!(::sycamore::component::AttributeValue::DangerouslySetInnerHtml(#value.to_string())),
                    )
                } else {
                    Ok(
                        quote!(::sycamore::component::AttributeValue::DynamicDangerouslySetInnerHtml(Box::new(#value))),
                    )
                }
            } else if is_bool_attr(&ident.to_string()) {
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
            ident,
            format!("unknown directive `{}`", ident),
        )),
    }
}

pub fn is_bool_attr(name: &str) -> bool {
    // Boolean attributes list from the WHATWG attributes table:
    // https://html.spec.whatwg.org/multipage/indices.html#attributes-3
    static BOOLEAN_ATTRIBUTES_SET: Lazy<HashSet<&str>> = Lazy::new(|| {
        vec![
            "allowfullscreen",
            "async",
            "autofocus",
            "autoplay",
            "checked",
            "controls",
            "default",
            "defer",
            "disabled",
            "formnovalidate",
            "hidden",
            "inert",
            "ismap",
            "itemscope",
            "loop",
            "multiple",
            "muted",
            "nomodule",
            "novalidate",
            "open",
            "playsinline",
            "readonly",
            "required",
            "reversed",
            "selected",
        ]
        .into_iter()
        .collect()
    });
    BOOLEAN_ATTRIBUTES_SET.contains(name)
}

fn is_component(ident: &TagIdent) -> bool {
    match ident {
        TagIdent::Path(path) => {
            path.get_ident().is_none()
                || path
                    .get_ident()
                    .unwrap()
                    .to_string()
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_uppercase()
        }
        // A hyphenated tag is always a custom-element and therefore never a component.
        TagIdent::Hyphenated(_) => false,
    }
}

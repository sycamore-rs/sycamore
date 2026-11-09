//! Codegen for `view!` macro.
//!
//! Implementation note: We are not using the `quote::ToTokens` trait because we need to pass
//! additional information to the codegen such as which mode (Client, Hydrate, SSR), etc...

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::Expr;
use syn::spanned::Spanned;

use crate::ir::{DynNode, Node, Prop, PropType, Root, TagIdent, TagNode, TextNode};

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
                    ::sycamore::rt::Build::<::sycamore::rt::View>::build(::std::vec![#(#nodes),*])
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
                ::sycamore::rt::Build::<::sycamore::rt::View>::build(#value)
            },
            Node::Dyn(DynNode { value }) => quote! {
                ::sycamore::rt::Build::<::sycamore::rt::View>::build(#value)
            },
            Node::If(if_node) => {
                let cond = &if_node.cond;
                let is_dynamic = is_closure(cond);

                let if_root = self.root(&if_node.then);
                let else_root = if_node
                    .else_branch
                    .as_ref()
                    .map(|else_branch| self.root(else_branch))
                    .unwrap_or_else(|| quote! { ::sycamore::rt::View::new() });

                if is_dynamic {
                    let condition =
                        syn::Ident::new("__sycamore_condition", proc_macro2::Span::mixed_site());
                    quote! {{
                        let mut #condition = #cond;
                        ::sycamore::rt::View::from_dynamic(move || {
                            if (&mut #condition)() {
                                #if_root
                            } else {
                                #else_root
                            }
                        })
                    }}
                } else {
                    quote! {
                        if #cond {
                            #if_root
                        } else {
                            #else_root
                        }
                    }
                }
            }
            Node::Match(match_node) => {
                let match_expr = &match_node.expr;
                let is_dynamic = is_closure(match_expr);

                let arms = match_node.arms.iter().map(|arm| {
                    let pat = &arm.pat;
                    let arm_root = self.root(&arm.body);
                    quote! {
                        #pat => #arm_root
                    }
                });

                if is_dynamic {
                    let scrutinee =
                        syn::Ident::new("__sycamore_scrutinee", proc_macro2::Span::mixed_site());
                    quote! {{
                        let mut #scrutinee = #match_expr;
                        ::sycamore::rt::View::from_dynamic(move || {
                            match (&mut #scrutinee)() {
                                #(#arms),*
                            }
                        })
                    }}
                } else {
                    quote! {
                        match #match_expr {
                            #(#arms),*
                        }
                    }
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
        // Like component props, element attributes are passed as-is. Reactivity is
        // opt-in via an explicit closure (or a reactive value such as a signal).
        let value = &attr.value;
        match &attr.ty {
            PropType::Plain { ident } => {
                quote! { .#ident(#value) }
            }
            PropType::PlainHyphenated { ident } => {
                quote! { .attr(#ident, #value) }
            }
            PropType::PlainQuoted { ident } => {
                quote! { .attr(#ident, #value) }
            }
            PropType::Directive { dir, ident } => match dir.to_string().as_str() {
                "prop" => {
                    let ident = ident.to_string();
                    quote! { .prop(#ident, #value) }
                }
                _ => syn::Error::new(dir.span(), format!("unknown directive `{dir}`"))
                    .to_compile_error(),
            },
            PropType::Ref => quote! { .r#ref(#value) },
            PropType::Spread => quote! { .spread(#value) },
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

        let other_props = props
            .iter()
            .filter(|prop| !matches!(&prop.ty, PropType::Plain { .. }))
            .collect::<Vec<_>>();
        let other_attributes = other_props.iter().map(|prop| self.attribute(prop));

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
        let call = quote_spanned!(ident.span()=> #ident());
        let build = quote_spanned!(ident.span()=> .build());
        quote! {{
            #call
                #(.#plain_names(#plain_values))*
                #(#other_attributes)*
                #children_quoted
                #build
        }}
    }
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

/// Only explicit closures opt DSL control flow into reactivity. Parentheses and
/// invisible groups do not change the meaning of a closure.
fn is_closure(expr: &Expr) -> bool {
    match expr {
        Expr::Closure(_) => true,
        Expr::Paren(expr) => is_closure(&expr.expr),
        Expr::Group(expr) => is_closure(&expr.expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_expressions_are_passed_directly() {
        for value in [
            quote! { signal.get() },
            quote! { move || signal.get() },
            quote! { format!("{}", signal.get()) },
            quote! { { let value = signal.get(); value + 1 } },
        ] {
            let value: Expr = syn::parse2(value).unwrap();
            let root: Root = syn::parse2(quote! { (#value) }).unwrap();
            let expected = quote! {
                ::sycamore::rt::Build::<::sycamore::rt::View>::build(#value)
            };
            assert_eq!(Codegen {}.root(&root).to_string(), expected.to_string());
        }
    }

    #[test]
    fn control_flow_reactivity_requires_an_explicit_closure() {
        for (source, dynamic) in [
            (quote! { if signal.get() { "yes" } else { "no" } }, false),
            (
                quote! { if move || signal.get() { "yes" } else { "no" } },
                true,
            ),
            (quote! { if (move || signal.get()) { "yes" } }, true),
            (quote! { if || true { "yes" } }, true),
            (
                quote! { match signal.get() { true => "yes", false => "no" } },
                false,
            ),
            (
                quote! { match move || signal.get() { true => "yes", false => "no" } },
                true,
            ),
            (
                quote! { match (move || signal.get()) { true => "yes", false => "no" } },
                true,
            ),
            (
                quote! { match true { _ if signal.get() => "yes", _ => "no" } },
                false,
            ),
        ] {
            let root: Root = syn::parse2(source.clone()).unwrap();
            let generated = Codegen {}.root(&root).to_string();
            assert_eq!(generated.contains("from_dynamic"), dynamic, "{source}");
        }
    }

    #[test]
    fn attribute_expressions_are_passed_directly() {
        for value in [quote! { signal.get() }, quote! { move || signal.get() }] {
            let value: Expr = syn::parse2(value).unwrap();
            let root: Root = syn::parse2(quote! {
                input(
                    value=#value,
                    data-value=#value,
                    "quoted-value"=#value,
                    prop:value=#value,
                )
            })
            .unwrap();
            let Node::Tag(tag) = &root.0[0] else {
                panic!("expected element");
            };
            let actual: TokenStream = tag
                .props
                .iter()
                .map(|prop| Codegen {}.attribute(prop))
                .collect();
            let expected = quote! {
                .value(#value)
                .attr("data-value", #value)
                .attr("quoted-value", #value)
                .prop("value", #value)
            };
            assert_eq!(actual.to_string(), expected.to_string());
        }
    }
}

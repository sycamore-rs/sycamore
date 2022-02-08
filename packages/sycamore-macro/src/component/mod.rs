//! The `#[component]` attribute macro implementation.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parse_quote, Expr, FnArg, Item, ItemFn, Pat, Result, ReturnType, Signature, Type, TypeTuple,
};

pub struct ComponentFunction {
    pub f: ItemFn,
}

impl Parse for ComponentFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed: Item = input.parse()?;

        match parsed {
            Item::Fn(mut f) => {
                let ItemFn { sig, .. } = &mut f;

                if sig.constness.is_some() {
                    return Err(syn::Error::new(
                        sig.constness.span(),
                        "const functions can't be components",
                    ));
                }

                if sig.abi.is_some() {
                    return Err(syn::Error::new(
                        sig.abi.span(),
                        "extern functions can't be components",
                    ));
                }

                if let ReturnType::Default = sig.output {
                    return Err(syn::Error::new(
                        sig.span(),
                        "function must return `sycamore::view::View`",
                    ));
                };

                let inputs = sig.inputs.clone().into_iter().collect::<Vec<_>>();

                if inputs.is_empty() {
                    return Err(syn::Error::new(
                        sig.inputs.span(),
                        "component must take at least one argument of type `sycamore::reactive::ScopeRef`",
                    ));
                }

                if inputs.len() > 2 {
                    return Err(syn::Error::new(
                        sig.inputs.span(),
                        "component should not take more than 2 arguments",
                    ));
                }

                if let FnArg::Receiver(arg) = &inputs[0] {
                    return Err(syn::Error::new(
                        arg.span(),
                        "function components can't accept a receiver",
                    ));
                }

                if let Some(FnArg::Typed(pat)) = inputs.get(1) {
                    if let Type::Tuple(TypeTuple { elems, .. }) = &*pat.ty {
                        if elems.is_empty() {
                            return Err(syn::Error::new(
                                elems.span(),
                                "taking an unit tuple as props is useless",
                            ));
                        }
                    }
                }

                // If only 1 argument, add an additional argument of type `()`.
                if inputs.len() == 1 {
                    sig.inputs.push(parse_quote! { _: () });
                }

                Ok(Self { f })
            }
            item => Err(syn::Error::new_spanned(
                item,
                "`component` attribute can only be applied to functions",
            )),
        }
    }
}

impl ToTokens for ComponentFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ComponentFunction { f } = self;
        let ItemFn {
            attrs,
            vis,
            sig,
            block,
        } = &f;

        if sig.asyncness.is_some() {
            let inputs = &sig.inputs;
            let args: Vec<Expr> = inputs
                .iter()
                .map(|x| match x {
                    FnArg::Typed(t) => match &*t.pat {
                        Pat::Ident(id) => {
                            let id = &id.ident;
                            parse_quote! { #id }
                        }
                        Pat::Wild(_) => parse_quote!(()),
                        _ => panic!("unexpected pattern"), // TODO
                    },
                    FnArg::Receiver(_) => unreachable!(),
                })
                .collect::<Vec<_>>();
            let non_async_sig = Signature {
                asyncness: None,
                ..sig.clone()
            };
            let inner_ident = format_ident!("{}_inner", sig.ident);
            let inner_sig = Signature {
                ident: inner_ident.clone(),
                ..sig.clone()
            };
            let ctx = match inputs.first().unwrap() {
                FnArg::Typed(t) => match &*t.pat {
                    Pat::Ident(id) => &id.ident,
                    _ => unreachable!(),
                },
                FnArg::Receiver(_) => unreachable!(),
            };
            tokens.extend(quote! {
                #[allow(non_snake_case)]
                #(#attrs)*
                #vis #non_async_sig {
                    #[allow(non_snake_case)]
                    #inner_sig #block

                    let __dyn = #ctx.create_signal(::sycamore::view::View::empty());
                    let __view = ::sycamore::view! { #ctx, (__dyn.get().as_ref().clone()) };

                    ::sycamore::suspense::suspense_scope(#ctx, async move {
                        let __async_view = #inner_ident(#(#args),*).await;
                        __dyn.set(__async_view);
                    });

                    __view
                }
            });
        } else {
            tokens.extend(quote! {
                #[allow(non_snake_case)]
                #f
            });
        }
    }
}

pub fn component_impl(comp: ComponentFunction) -> Result<TokenStream> {
    Ok(comp.to_token_stream())
}

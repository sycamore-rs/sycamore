//! The `#[component]` attribute macro implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, FnArg, Item, ItemFn, Result, ReturnType, Type, TypeTuple};

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

pub fn component_impl(comp: ComponentFunction) -> Result<TokenStream> {
    let ComponentFunction { f } = comp;

    Ok(quote! {
        #[allow(non_snake_case)]
        #f
    })
}

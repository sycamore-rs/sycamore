//! The `#[component]` attribute macro implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{FnArg, Item, ItemFn, Result, ReturnType};

pub struct ComponentFunction {
    pub f: ItemFn,
}

impl Parse for ComponentFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed: Item = input.parse()?;

        match parsed {
            Item::Fn(f) => {
                let ItemFn { sig, .. } = f.clone();

                if sig.constness.is_some() {
                    return Err(syn::Error::new_spanned(
                        sig.constness,
                        "const functions can't be components",
                    ));
                }

                if sig.abi.is_some() {
                    return Err(syn::Error::new_spanned(
                        sig.abi,
                        "extern functions can't be components",
                    ));
                }

                if let ReturnType::Default = sig.output {
                    return Err(syn::Error::new_spanned(
                        sig,
                        "function must return `sycamore::view::View`",
                    ));
                };

                let mut inputs = sig.inputs.into_iter();
                let arg: FnArg = inputs.next().unwrap_or_else(|| syn::parse_quote! { _: () });

                if let FnArg::Receiver(arg) = &arg {
                    return Err(syn::Error::new_spanned(
                        arg,
                        "function components can't accept a receiver",
                    ));
                };

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

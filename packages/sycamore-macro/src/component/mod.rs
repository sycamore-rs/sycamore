//! The `#[component]` attribute macro implementation.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_quote, Expr, FnArg, Item, ItemFn, Pat, Result, ReturnType, Signature, Token, Type,
    TypeTuple,
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
                        sig.paren_token.span,
                        "component must return `sycamore::view::View`",
                    ));
                };

                let inputs = sig.inputs.clone().into_iter().collect::<Vec<_>>();

                if inputs.is_empty() {
                    return Err(syn::Error::new(
                        sig.paren_token.span,
                        "component must take at least one argument of type `sycamore::reactive::Scope`",
                    ));
                }

                if inputs.len() > 2 {
                    return Err(syn::Error::new(
                        sig.inputs
                            .clone()
                            .into_iter()
                            .skip(2)
                            .collect::<Punctuated<_, Token![,]>>()
                            .span(),
                        "component should not take more than 2 arguments",
                    ));
                }

                if let FnArg::Typed(t) = &inputs[0] {
                    if !matches!(&*t.pat, Pat::Ident(_)) {
                        return Err(syn::Error::new(
                            t.span(),
                            "First argument to a component is expected to be a `sycamore::reactive::Scope`",
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        inputs[0].span(),
                        "function components can't accept a receiver",
                    ));
                }

                if let Some(FnArg::Typed(pat)) = inputs.get(1) {
                    if let Type::Tuple(TypeTuple { elems, .. }) = &*pat.ty {
                        if elems.is_empty() {
                            return Err(syn::Error::new(
                                pat.ty.span(),
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
                "the `component` attribute can only be applied to functions",
            )),
        }
    }
}

struct AsyncCompInputs {
    cx: syn::Ident,
    sync_input: Punctuated<FnArg, syn::token::Comma>,
    async_args: Vec<Expr>,
}

fn async_comp_inputs_from_sig_inputs(
    inputs: &Punctuated<FnArg, syn::token::Comma>,
) -> AsyncCompInputs {
    let mut sync_input = Punctuated::new();
    let mut async_args = Vec::with_capacity(2);

    #[inline]
    fn pat_ident_arm(
        sync_input: &mut Punctuated<FnArg, syn::token::Comma>,
        fn_arg: &FnArg,
        id: &syn::PatIdent,
    ) -> syn::Expr {
        sync_input.push(fn_arg.clone());
        let ident = &id.ident;
        parse_quote! { #ident }
    }

    let mut inputs = inputs.iter();

    let cx_fn_arg = inputs.next().unwrap();

    let cx = if let FnArg::Typed(t) = cx_fn_arg {
        if let Pat::Ident(id) = &*t.pat {
            async_args.push(pat_ident_arm(&mut sync_input, cx_fn_arg, id));
            id.ident.clone()
        } else {
            unreachable!("We check in parsing that the first argument is a Ident");
        }
    } else {
        unreachable!("We check in parsing that the first argument is not a receiver");
    };

    // In parsing we checked that there were two args so we can unwrap here.
    let prop_fn_arg = inputs.next().unwrap();
    let prop_arg = match prop_fn_arg {
        FnArg::Typed(t) => match &*t.pat {
            Pat::Ident(id) => pat_ident_arm(&mut sync_input, prop_fn_arg, id),
            Pat::Wild(_) => {
                sync_input.push(prop_fn_arg.clone());
                parse_quote!(())
            }
            Pat::Struct(pat_struct) => {
                // For the sync input we don't want a destructured pattern but just to take a
                // `syn::PatType` (i.e. `props: MyPropStruct`) then the inner async function
                // signature can have the destructured pattern and it will work correctly
                // aslong as we provide our brand new ident that we used in the
                // `syn::PatIdent`.
                let ident = syn::Ident::new("props", pat_struct.span());
                // props are taken by value so no refs or mutability required here
                // The destructured pattern can add mutability (if required) even without this
                // set.
                let pat_ident = syn::PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident,
                    subpat: None,
                };
                let pat_type = syn::PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(pat_ident)),
                    colon_token: Default::default(),
                    ty: t.ty.clone(),
                };

                let fn_arg = FnArg::Typed(pat_type);
                sync_input.push(fn_arg);
                parse_quote! { props }
            }
            _ => panic!("unexpected pattern!"),
        },
        FnArg::Receiver(_) => unreachable!(),
    };

    async_args.push(prop_arg);

    AsyncCompInputs {
        cx,
        async_args,
        sync_input,
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
            // When the component function is async then we need to extract out some of the
            // function signature (Syn::Signature) so that we can wrap the async function with
            // a non-async component.
            //
            // In order to support the struct destructured pattern for props we alter the existing
            // signature for the non-async component so that it is defined as a `Syn::PatType`
            // (i.e. props: MyPropStruct) with a new `Syn::Ident` "props". We then use this ident
            // again as an argument to the inner async function which has the user defined
            // destructured pattern which will work as expected.
            //
            // Note: that the change to the signature is not semantically different to a would be
            // caller.
            let inputs = &sig.inputs;
            let AsyncCompInputs {
                cx,
                sync_input,
                async_args: args,
            } = async_comp_inputs_from_sig_inputs(inputs);

            let non_async_sig = Signature {
                asyncness: None,
                inputs: sync_input,
                ..sig.clone()
            };
            let inner_ident = format_ident!("{}_inner", sig.ident);
            let inner_sig = Signature {
                ident: inner_ident.clone(),
                ..sig.clone()
            };
            tokens.extend(quote! {
                #[allow(non_snake_case)]
                #(#attrs)*
                #vis #non_async_sig {
                    #[allow(non_snake_case)]
                    #inner_sig #block

                    let __dyn = ::sycamore::reactive::create_signal(#cx, ::sycamore::view::View::empty());
                    let __view = ::sycamore::view::View::new_dyn(#cx, || <_ as ::std::clone::Clone>::clone(&*__dyn.get()));

                    ::sycamore::suspense::suspense_scope(#cx, async move {
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

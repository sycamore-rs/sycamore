use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Block, FnArg, Generics, Ident, Item, ItemFn, Result, ReturnType, Type, TypeParam,
    Visibility,
};

pub struct ComponentFunctionName {
    pub component_name: Ident,
    pub generics: Generics,
}

impl Parse for ComponentFunctionName {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            Err(input.error("expected an identifier for the component"))
        } else {
            let component_name: Ident = input.parse()?;
            let generics: Generics = input.parse()?;

            if let Some(lifetime) = generics.lifetimes().next() {
                return Err(syn::Error::new_spanned(
                    lifetime,
                    "unexpected lifetime param; put lifetime params on function instead",
                ));
            }

            if let Some(const_param) = generics.const_params().next() {
                return Err(syn::Error::new_spanned(
                    const_param,
                    "unexpected const generic param; put const generic params on function instead",
                ));
            }

            if generics.type_params().count() != 1 {
                return Err(syn::Error::new_spanned(
                    generics,
                    "expected a single type param",
                ));
            }

            if !generics
                .type_params()
                .next()
                .unwrap()
                .bounds
                .empty_or_trailing()
            {
                return Err(syn::Error::new_spanned(
                    generics,
                    "unexpected type bound in generic type",
                ));
            }

            Ok(Self {
                component_name,
                generics,
            })
        }
    }
}

pub struct ComponentFunction {
    pub block: Box<Block>,
    pub props_type: Box<Type>,
    pub arg: FnArg,
    pub generics: Generics,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub return_type: Box<Type>,
}

impl Parse for ComponentFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed: Item = input.parse()?;

        match parsed {
            Item::Fn(func) => {
                let ItemFn {
                    attrs,
                    vis,
                    sig,
                    block,
                } = func;

                if sig.asyncness.is_some() {
                    return Err(syn::Error::new_spanned(
                        sig.asyncness,
                        "async functions can't be components",
                    ));
                }

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

                let return_type =
                    match sig.output {
                        ReturnType::Default => return Err(syn::Error::new_spanned(
                            sig,
                            "function must return `maple_core::template_result::TemplateResult`",
                        )),
                        ReturnType::Type(_, ty) => ty,
                    };

                let mut inputs = sig.inputs.into_iter();
                let arg: FnArg = inputs.next().unwrap_or_else(|| syn::parse_quote! { _: () });

                let props_type = match &arg {
                    FnArg::Typed(arg) => arg.ty.clone(),
                    FnArg::Receiver(arg) => {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "function components can't accept a receiver",
                        ))
                    }
                };

                if inputs.len() > 0 {
                    let params: TokenStream = inputs.map(|it| it.to_token_stream()).collect();
                    return Err(syn::Error::new_spanned(
                        params,
                        "function should accept at most one parameter for the prop",
                    ));
                }

                Ok(Self {
                    block,
                    props_type,
                    arg,
                    generics: sig.generics,
                    vis,
                    attrs,
                    name: sig.ident,
                    return_type,
                })
            }
            item => Err(syn::Error::new_spanned(
                item,
                "`function_component` attribute can only be applied to functions",
            )),
        }
    }
}

pub fn component_impl(
    attr: ComponentFunctionName,
    component: ComponentFunction,
) -> Result<TokenStream> {
    let ComponentFunctionName {
        component_name,
        generics: generic_node_ty,
    } = attr;

    let component_name_str = component_name.to_string();
    let generic_node_ty = generic_node_ty.type_params().next().unwrap();
    let generic_node: TypeParam = syn::parse_quote! {
        #generic_node_ty: ::maple_core::generic_node::GenericNode
    };

    let ComponentFunction {
        block,
        props_type: _,
        arg,
        generics,
        vis,
        attrs,
        name,
        return_type,
    } = component;

    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    if name == component_name {
        return Err(syn::Error::new_spanned(
            component_name,
            "the component must not have the same name as the function",
        ));
    }

    let quoted = quote! {
        #(#attrs)*
        #vis struct #component_name<#generic_node> {
            #[doc(hidden)]
            _marker: ::std::marker::PhantomData<#generic_node_ty>,
        }

        impl<#generic_node> ::maple_core::component::Component<#generic_node_ty>
            for #component_name<#generic_node_ty>
        {
            #[cfg(debug_assertions)]
            const NAME: &'static ::std::primitive::str = #component_name_str;
        }

        impl<#generic_node> #component_name<#generic_node_ty> {
            #[doc(hidden)]
            pub fn __create_component#impl_generics(#arg) -> #return_type
                #where_clause
            {
                #block
            }
        }
    };

    Ok(quoted)
}

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Fields, Ident, LitStr, Token, Variant};

use crate::parser::route;
use crate::parser::RoutePathAst;
use crate::parser::SegmentAst;

pub fn route_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let mut quoted = TokenStream::new();
    let mut err_quoted = TokenStream::new();
    let mut has_error_handler = false;

    match &input.data {
        syn::Data::Enum(de) => {
            let ty_name = &input.ident;

            for variant in &de.variants {
                let variant_id = &variant.ident;

                let has_preload_handler = variant
                    .attrs
                    .iter()
                    .any(|attr| *attr.path.get_ident().unwrap() == "preload");

                let mut quote_match_route = None;
                let mut quote_prefetch = None;

                for attr in &variant.attrs {
                    let attr_name = match attr.path.get_ident() {
                        Some(ident) => ident.to_string(),
                        None => continue,
                    };

                    match attr_name.as_str() {
                        "to" => {
                            quote_match_route =
                                Some(impl_to(attr, variant, variant_id, has_preload_handler)?);
                        }
                        "not_found" => {
                            if has_error_handler {
                                return Err(syn::Error::new(
                                    attr.span(),
                                    "cannot have more than one error handler",
                                ));
                            }
                            if !variant.fields.is_empty() {
                                return Err(syn::Error::new(
                                    variant.fields.span(),
                                    "not found route cannot have any fields",
                                ));
                            }
                            err_quoted = quote! {
                                return Self::#variant_id;
                            };
                            has_error_handler = true;
                        }
                        "preload" => {
                            quote_prefetch = Some(quote! {});
                        }
                        _ => {}
                    }
                }
                if let Some(quote_prefetch) = quote_prefetch {
                    quoted.extend(quote_prefetch);
                }
                quoted.extend(quote_match_route);
            }

            if !has_error_handler {
                return Err(syn::Error::new(
                    input.span(),
                    "not found route not specified",
                ));
            }

            Ok(quote! {
                #[::sycamore_router::rt::async_trait]
                impl ::sycamore_router::Route for #ty_name {
                    async fn match_route(path: &[&str]) -> Self {
                        #quoted
                        #err_quoted
                    }
                }
            })
        }
        _ => Err(syn::Error::new(
            input.span(),
            "Route can only be derived on enums",
        )),
    }
}

/// Implementation for `#[to(_)]` attribute.
fn impl_to(
    attr: &Attribute,
    variant: &Variant,
    variant_id: &Ident,
    has_preload_handler: bool,
) -> Result<TokenStream, syn::Error> {
    // region: parse route
    let route_litstr: LitStr = attr.parse_args()?;
    let route_str = route_litstr.value();
    let route = match route(&route_str) {
        Ok(("", route_ast)) => route_ast,
        Ok((_, _)) => unreachable!("parser error"),
        Err(_err) => {
            return Err(syn::Error::new(route_litstr.span(), "route is malformed"));
        }
    };
    // endregion

    let dyn_segments = route.dyn_segments();
    let expected_fields_len = if has_preload_handler {
        dyn_segments.len() + 1
    } else {
        dyn_segments.len()
    };
    if expected_fields_len != variant.fields.len() {
        if has_preload_handler && dyn_segments.len() == variant.fields.len() {
            return Err(syn::Error::new(
                variant.fields.span(),
                "missing field for preload data",
            ));
        } else {
            return Err(syn::Error::new(
                variant.fields.span(),
                "mismatch between number of capture fields and variant fields",
            ));
        }
    }

    let capture_vars = match &variant.fields {
        // For named fields, captures must match the field name.
        Fields::Named(f) => {
            let mut captures = Vec::new();

            for (i, (field, segment)) in f.named.iter().zip(dyn_segments.iter()).enumerate() {
                let field_ty = &field.ty;
                match segment {
                    SegmentAst::Param(_) => unreachable!("not a dynamic segment"),
                    SegmentAst::DynParam(param) => {
                        if param != &field.ident.as_ref().unwrap().to_string() {
                            return Err(syn::Error::new(
                                field.ident.span(),
                                "capture field name mismatch",
                            ));
                        }
                        let param_id: Ident = syn::parse_str(param)?;
                        captures.push(quote! {
                            let mut #param_id = <#field_ty as ::std::default::Default>::default();
                            if !::sycamore_router::FromParam::set_value(
                                &mut #param_id,
                                captures[#i].as_dyn_param().unwrap()
                            ) {
                                break;
                            }
                        })
                    }
                    SegmentAst::DynSegments(param) => {
                        if param != &field.ident.as_ref().unwrap().to_string() {
                            return Err(syn::Error::new(
                                field.ident.span(),
                                "capture field name mismatch",
                            ));
                        }
                        let param_id: Ident = syn::parse_str(param)?;
                        captures.push(quote! {
                            let mut #param_id = <#field_ty as ::std::default::Default>::default();
                            if !::sycamore_router::FromSegments::set_value(
                                &mut #param_id,
                                captures[#i].as_dyn_segments().unwrap()
                            ) {
                                break;
                            }
                        })
                    }
                }
            }
            let mut named: Punctuated<&Option<Ident>, Token![,]> =
                f.named.iter().map(|x| &x.ident).collect();
            if has_preload_handler {
                if f.named.last().unwrap().ident.as_ref().unwrap() != "data" {
                    return Err(syn::Error::new(
                        f.named.last().unwrap().span(),
                        "preload field must be named `data`",
                    ));
                }

                captures.push(quote! {
                    let data = todo!();
                });
                named.push(&f.named.last().unwrap().ident);
            }
            quote! {
                loop {
                    #(#captures)*
                    return Self::#variant_id {
                        #named
                    };
                }
            }
        }
        // For unnamed fields, captures must be in right order.
        Fields::Unnamed(fu) => {
            let mut captures = Vec::new();

            for (i, (field, segment)) in fu.unnamed.iter().zip(dyn_segments.iter()).enumerate() {
                let field_ty = &field.ty;
                match segment {
                    SegmentAst::Param(_) => unreachable!("not a dynamic segment"),
                    SegmentAst::DynParam(_) => captures.push(quote! {{
                        let mut value = <#field_ty as ::std::default::Default>::default();
                        if ::sycamore_router::FromParam::set_value(
                            &mut value,
                            captures[#i].as_dyn_param().unwrap()
                        ) {
                            value
                        } else {
                            break;
                        }
                    }}),
                    SegmentAst::DynSegments(_) => captures.push(quote! {{
                        let mut value = <#field_ty as ::std::default::Default>::default();
                        if ::sycamore_router::FromSegments::set_value(
                            &mut value,
                            captures[#i].as_dyn_segments().unwrap()
                        ) {
                            value
                        } else {
                            break;
                        }
                    }}),
                }
            }
            quote! {
                // Run captures inside a loop in order to allow early break inside the expression.
                loop {
                    return Self::#variant_id(#(#captures),*);
                }
            }
        }
        Fields::Unit => quote! {
            return Self::#variant_id;
        },
    };
    Ok(quote! {
        let route = #route;
        if let Some(captures) = route.match_path(path) {
            // Try to capture variables.
            #capture_vars
        }
    })
}

impl<'a> ToTokens for SegmentAst<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SegmentAst::Param(param) => tokens.extend(quote! {
                ::sycamore_router::Segment::Param(::std::string::ToString::to_string(#param))
            }),
            SegmentAst::DynParam(_) => tokens.extend(quote! {
                ::sycamore_router::Segment::DynParam
            }),
            SegmentAst::DynSegments(_) => tokens.extend(quote! {
                ::sycamore_router::Segment::DynSegments
            }),
        }
    }
}

impl<'a> ToTokens for RoutePathAst<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let segments = self
            .segments
            .iter()
            .map(|s| s.to_token_stream())
            .collect::<Vec<_>>();

        tokens.extend(quote! {
            ::sycamore_router::RoutePath::new(::std::vec![#(#segments),*])
        });
    }
}

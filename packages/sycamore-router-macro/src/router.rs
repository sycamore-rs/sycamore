use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::Fields;
use syn::LitStr;

use crate::parser::route;
use crate::parser::RouteAst;
use crate::parser::SegmentAst;

pub fn router_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let mut quoted = TokenStream::new();
    let mut err_quoted = TokenStream::new();
    let mut has_error_handler = false;

    match &input.data {
        syn::Data::Enum(de) => {
            let ty_name = &input.ident;

            for variant in &de.variants {
                let variant_id = &variant.ident;

                for attr in &variant.attrs {
                    let attr_name = match attr.path.get_ident() {
                        Some(ident) => ident.to_string(),
                        None => continue,
                    };

                    match attr_name.as_str() {
                        "to" => {
                            let route_litstr: LitStr = attr.parse_args()?;
                            let route_str = route_litstr.value();

                            let (remaining, route) = match route(&route_str) {
                                Ok(route_ast) => route_ast,
                                Err(_err) => {
                                    return Err(syn::Error::new(
                                        route_litstr.span(),
                                        "route is malformed",
                                    ));
                                }
                            };
                            assert!(remaining.is_empty());

                            let dyn_segments = route.dyn_segments();

                            if dyn_segments.len() != variant.fields.len() {
                                return Err(syn::Error::new(
                                    variant.fields.span(),
                                    "mismatch between number of capture fields and variant fields",
                                ));
                            }

                            let capture_vars = match &variant.fields {
                                Fields::Named(_) => todo!(),
                                Fields::Unnamed(fu) => {
                                    let mut captures = Vec::new();

                                    for (i, (field, segment)) in
                                        fu.unnamed.iter().zip(dyn_segments.iter()).enumerate()
                                    {
                                        let field_ty = &field.ty;
                                        match segment {
                                            SegmentAst::Param(_) => unreachable!(),
                                            SegmentAst::DynParam(_) => captures.push(quote! {{
                                                let mut value = <#field_ty as ::std::default::Default>::default();
                                                if ::sycamore_router::FromParam::set_value(
                                                    &mut value,
                                                    captures[#i],
                                                ) {
                                                    value
                                                } else {
                                                    break;
                                                }
                                            }}),
                                            SegmentAst::DynSegments(_) => todo!(),
                                        }
                                    }
                                    quote! {
                                        loop {
                                            return Self::#variant_id(#(#captures),*);
                                        }
                                    }
                                }
                                Fields::Unit => quote! {
                                    return Self::#variant_id;
                                },
                            };

                            quoted.extend(quote! {
                                let route = #route;
                                if let Some(captures) = route.match_path(path) {
                                    // Try to capture variables.
                                    #capture_vars
                                }
                            });
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
                        _ => {}
                    }
                }
            }

            if !has_error_handler {
                return Err(syn::Error::new(
                    input.span(),
                    "not found route not specified",
                ));
            }

            Ok(quote! {
                impl ::sycamore_router::Router for #ty_name {
                    fn match_route(path: &[&str]) -> Self {
                        #quoted
                        #err_quoted
                    }
                }
            })
        }
        _ => Err(syn::Error::new(
            input.span(),
            "Router can only be derived on enums",
        )),
    }
}

impl<'a> ToTokens for SegmentAst<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SegmentAst::Param(param) => tokens.extend(quote! {
                ::sycamore_router::Segment::Param(#param)
            }),
            SegmentAst::DynParam(_) => tokens.extend(quote! {
                ::sycamore_router::DynParam
            }),
            SegmentAst::DynSegments(_) => tokens.extend(quote! {
                ::sycamore_router::DynSegments
            }),
        }
    }
}

impl<'a> ToTokens for RouteAst<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let segments = self
            .segments
            .iter()
            .map(|s| s.to_token_stream())
            .collect::<Vec<_>>();

        tokens.extend(quote! {
            ::sycamore_router::Route::new(::std::vec![#(#segments),*])
        });
    }
}

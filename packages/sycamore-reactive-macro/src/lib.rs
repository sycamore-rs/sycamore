use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, Result};

#[proc_macro_derive(State, attributes(nested))]
pub fn state(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    impl_state(&ast)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn impl_state(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    match &ast.data {
        syn::Data::Struct(data) => impl_state_struct(ast, data),
        _ => Err(syn::Error::new_spanned(
            ast,
            "State can only be derived for structs",
        )),
    }
}

/// Create a new struct that mirrors all the fields of the original struct but with the type
/// replaced with a reactive trigger.
///
/// If the field is a nested field, then use the trigger struct of the nested field instead of a
/// trigger signal.
fn impl_state_struct(
    ast: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> Result<proc_macro2::TokenStream> {
    let vis = &ast.vis;
    let ident = &ast.ident;
    let trigger_ident = format_ident!("{}__Trigger", ident);

    let leaf_fields = data
        .fields
        .iter()
        .filter(|f| !f.attrs.iter().any(|attr| attr.path().is_ident("nested")));
    let leaf_idents = leaf_fields.map(|f| &f.ident).collect::<Vec<_>>();

    let node_fields = data
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|attr| attr.path().is_ident("nested")));
    let node_idents = node_fields.clone().map(|f| &f.ident).collect::<Vec<_>>();
    let node_types = node_fields.map(|f| &f.ty).collect::<Vec<_>>();

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    Ok(quote! {
        #[doc = "A mirrored version of [`"]
        #[doc = ::std::stringify!(#ident)]
        #[doc = "`] with triggers for all the fields."]
        #[allow(non_camel_case_types)]
        #vis struct #trigger_ident #ty_generics #where_clause {
            #(#leaf_idents: ::sycamore_reactive3::Signal<()>,)*
            #(#node_idents: <#node_types as ::sycamore_reactive3::State>::Trigger,)*
        }

        impl #impl_generics #trigger_ident #ty_generics #where_clause {
            pub fn new(cx: ::sycamore_reactive3::Scope) -> Self {
                Self {
                    #(#leaf_idents: ::sycamore_reactive3::create_signal(cx, ()),)*
                    #(#node_idents: <#node_types as ::sycamore_reactive3::State>::Trigger::new(cx),)*
                }
            }
        }

        impl ::sycamore_reactive3::State for #impl_generics #ident #ty_generics #where_clause {
            type Trigger = #trigger_ident;
        }
    })
}

struct LensPath {
    first: Ident,
    segments: Vec<LensSegment>,
}

enum LensSegment {
    Field(Ident),
}

impl Parse for LensPath {
    fn parse(input: ParseStream) -> Result<Self> {
        let first = input.parse()?;
        let mut segments = Vec::new();
        while !input.is_empty() {
            segments.push(input.parse()?);
        }
        Ok(LensPath { first, segments })
    }
}

impl Parse for LensSegment {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<syn::Token![.]>()?;
        let ident = input.parse()?;
        Ok(LensSegment::Field(ident))
    }
}

#[proc_macro]
pub fn get(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path = parse_macro_input!(input as LensPath);
    let mut tokens = path.first.to_token_stream();
    for segment in path.segments {
        match segment {
            LensSegment::Field(ident) => {
                tokens.extend(quote!(.#ident));
            }
        }
    }
    tokens.into()
}

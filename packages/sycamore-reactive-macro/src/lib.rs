use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::token::Bracket;
use syn::{parse_macro_input, Expr, Ident, Result, Token};

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

        impl #impl_generics ::sycamore_reactive3::StateTrigger for #trigger_ident #ty_generics #where_clause {
            fn new(cx: ::sycamore_reactive3::Scope) -> Self {
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
        while input.peek(Token![.]) || input.peek(Bracket) {
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

impl LensPath {
    fn to_value_path(&self) -> TokenStream {
        let mut tokens = TokenStream::default();
        for segment in &self.segments {
            match segment {
                LensSegment::Field(ident) => {
                    tokens.extend(quote!(.#ident));
                }
            }
        }
        tokens
    }

    fn to_trigger_path(&self) -> TokenStream {
        let mut tokens = TokenStream::default();
        for segment in &self.segments {
            match segment {
                LensSegment::Field(ident) => {
                    tokens.extend(quote!(.#ident));
                }
            }
        }
        tokens
    }
}

#[proc_macro]
pub fn get(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path = parse_macro_input!(input as LensPath);

    let value_path = path.to_value_path();
    let trigger_path = path.to_trigger_path();
    let first = path.first;

    quote! {{
        // Track the value.
        ::sycamore_reactive3::Store::__trigger(&#first) #trigger_path.get();

        ::sycamore_reactive3::Store::__with(&#first, |#first| #first #value_path)
    }}
    .into()
}

struct SetInput {
    path: LensPath,
    value: Expr,
}

impl Parse for SetInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        input.parse::<Token![,]>()?;
        let value = input.parse()?;
        Ok(SetInput { path, value })
    }
}

#[proc_macro]
pub fn set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let SetInput { path, value } = parse_macro_input!(input as SetInput);

    let value_path = path.to_value_path();
    let trigger_path = path.to_trigger_path();
    if path.segments.is_empty() {
        return syn::Error::new_spanned(path.first, "Cannot use `set!` on the root of a store.")
            .to_compile_error()
            .into();
    }

    let first = path.first;

    quote! {{
        ::sycamore_reactive3::Store::__with_mut(&#first, |#first| #first #value_path = #value);
        ::sycamore_reactive3::Store::__trigger(&#first) #trigger_path.set(());
    }}
    .into()
}

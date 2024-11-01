use proc_macro2::Span;
use quote::format_ident;
use syn::punctuated::Punctuated;
use syn::{
    Field, GenericParam, Generics, Ident, Path, PathArguments, PathSegment, Token, Type,
    TypeImplTrait, TypeParam, TypePath, Visibility,
};

pub fn create_generic_ident(generics: &Generics) -> Ident {
    format_ident!("__T{}", generics.params.len())
}

pub fn resolve_type(generics: &mut Generics, ty: Type) -> Type {
    match ty {
        Type::ImplTrait(inner) => add_generic(generics, inner),
        Type::Array(inner) => {
            let elem = resolve_type(generics, *inner.elem);
            Type::Array(syn::TypeArray {
                bracket_token: inner.bracket_token,
                elem: Box::new(elem),
                semi_token: inner.semi_token,
                len: inner.len,
            })
        }
        Type::Paren(inner) => {
            let elem = resolve_type(generics, *inner.elem);
            Type::Paren(syn::TypeParen {
                paren_token: inner.paren_token,
                elem: Box::new(elem),
            })
        }
        Type::Ptr(inner) => {
            let elem = resolve_type(generics, *inner.elem);
            Type::Ptr(syn::TypePtr {
                star_token: inner.star_token,
                const_token: inner.const_token,
                mutability: inner.mutability,
                elem: Box::new(elem),
            })
        }
        Type::Reference(inner) => {
            let elem = resolve_type(generics, *inner.elem);
            Type::Reference(syn::TypeReference {
                and_token: inner.and_token,
                lifetime: inner.lifetime,
                mutability: inner.mutability,
                elem: Box::new(elem),
            })
        }
        Type::Slice(inner) => {
            let elem = resolve_type(generics, *inner.elem);
            Type::Slice(syn::TypeSlice {
                bracket_token: inner.bracket_token,
                elem: Box::new(elem),
            })
        }
        Type::Tuple(inner) => {
            let elems = inner
                .elems
                .iter()
                .map(|elem| resolve_type(generics, elem.clone()))
                .collect();
            Type::Tuple(syn::TypeTuple {
                paren_token: inner.paren_token,
                elems,
            })
        }
        _ => ty,
    }
}

pub fn add_generic(generics: &mut Generics, impl_type: TypeImplTrait) -> Type {
    let type_ident = create_generic_ident(generics);
    let type_param = TypeParam {
        attrs: Vec::new(),
        ident: type_ident.clone(),
        colon_token: Some(Token![:](Span::call_site())),
        bounds: impl_type.bounds,
        eq_token: None,
        default: None,
    };

    generics.params.push(GenericParam::Type(type_param));

    Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: Punctuated::from_iter(vec![PathSegment {
                ident: type_ident,
                arguments: PathArguments::None,
            }]),
        },
    })
}

pub fn push_field(fields: &mut Vec<Field>, generics: &mut Generics, ident: Ident, ty: Type) {
    let ty = resolve_type(generics, ty);

    fields.push(Field {
        attrs: Vec::new(),
        vis: Visibility::Public(Token![pub](Span::call_site())),
        mutability: syn::FieldMutability::None,
        ident: Some(ident),
        ty,
        colon_token: Some(Token![:](Span::call_site())),
    });
}

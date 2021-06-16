use proc_macro2::TokenStream;
use syn::LitStr;
use syn::spanned::Spanned;
use syn::DeriveInput;

pub fn router_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let mut has_error_handler = false;

    match input.data {
        syn::Data::Enum(de) => {
            for variant in de.variants {
                for attr in variant.attrs {
                    let attr_name = match attr.path.get_ident() {
                        Some(ident) => ident.to_string(),
                        None => continue,
                    };

                    match attr_name.as_str() {
                        "to" => {
                            let route_litstr: LitStr = attr.parse_args()?;
                            let route_str = route_litstr.value();
                        },
                        "err" => {
                            if has_error_handler {
                                return Err(syn::Error::new(
                                    attr.span(),
                                    "Cannot have more than one error handler",
                                ));
                            }
                            todo!();
                            has_error_handler = true;
                        }
                        _ => {}
                    }
                }
            }

            todo!()
        }
        _ => Err(syn::Error::new(
            input.span(),
            "Router can only be derived on enums",
        )),
    }
}

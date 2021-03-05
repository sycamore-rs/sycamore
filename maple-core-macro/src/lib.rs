mod element;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as element::HtmlTag);

    let quoted = quote::quote! {
        #input
    };

    TokenStream::from(quoted)
}

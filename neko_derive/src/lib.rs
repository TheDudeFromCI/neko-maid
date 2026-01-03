use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};

#[proc_macro_derive(NekoMarker, attributes(neko_marker))]
pub fn derive_neko_marker(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    // Find #[neko_marker("...")]
    let mut marker_value: Option<LitStr> = None;

    for attr in &input.attrs {
        if attr.path().is_ident("neko_marker") {
            let lit: LitStr = attr
                .parse_args()
                .expect("neko_marker expects a string literal");
            marker_value = Some(lit);
        }
    }

    let marker_value = marker_value
        .expect("Missing #[neko_marker(\"...\")] attribute");

    let expanded = quote! {
        impl NekoMarker for #ident {
            fn new() -> Self {
                Self
            }

            fn id() -> &'static str {
                #marker_value
            }
        }
    };

    expanded.into()
}

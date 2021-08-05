extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Responder)]
pub fn derive_responder(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;

    let gen = quote! {
        impl Responder for #name {

            fn respond_to(self, _req: &HttpRequest) -> HttpResponse {
                let body = serde_json::to_string(&self).unwrap();
                // create response and set content type
                HttpResponse::Ok().content_type(ContentType::json()).body(Body::from(body))
            }
        }
    };
    gen.into()
}

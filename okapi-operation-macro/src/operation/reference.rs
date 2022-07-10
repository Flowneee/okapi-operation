use darling::{FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) static REFERENCE_ATTRIBUTE_NAME: &str = "reference";

#[derive(Debug, FromMeta)]
pub(super) struct Reference(pub String);

impl Reference {
    pub fn name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or_default()
    }
}

impl ToTokens for Reference {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let reference = self.0.as_str();
        tokens.extend(quote! {
            okapi::openapi3::RefOr::Ref(okapi::openapi3::Ref {
                reference: #reference.into()
            })
        });
    }
}

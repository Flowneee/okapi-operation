use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::utils::quote_option;

#[derive(Debug, FromMeta)]
pub(super) struct ExternalDocs {
    url: String,
    #[darling(default)]
    description: Option<String>,
}

impl ToTokens for ExternalDocs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let url = &self.url;
        let description = quote_option(&self.description);
        tokens.extend(quote! {
            okapi::openapi3::ExternalDocs {
                description: #description,
                url: #url.into(),
                ..Default::default()
            }
        })
    }
}

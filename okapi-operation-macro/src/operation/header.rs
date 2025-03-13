use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Path;

use crate::{operation::parameters::ParameterStyle, utils::quote_option};

pub(super) static HEADER_ATTRIBUTE_NAME: &str = "header";

/// Header common description (in both `parameters` and `responses` sections).
#[derive(Debug, FromMeta)]
pub(super) struct Header {
    pub name: String,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    required: bool,
    #[darling(default)]
    deprecated: bool,
    #[darling(default)]
    style: Option<ParameterStyle>,
    schema: Path,
    // TODO: support content as well
}

impl Header {
    fn schema(&self) -> TokenStream {
        let style = quote_option(&self.style);
        let ty = &self.schema;
        quote! {
            okapi::openapi3::ParameterValue::Schema {
                style: #style,
                explode: None,
                allow_reserved: false,
                schema: components.schema_for::<#ty>(),
                example: Default::default(),
                examples: Default::default(),
            }
        }
    }

    pub(super) fn for_parameter(&self) -> ParameterHeader {
        ParameterHeader(self)
    }

    pub(super) fn for_response(&self) -> ResponseHeader {
        ResponseHeader(self)
    }
}

/// Wrapper, implementing [`ToTokens`] for [`Parameter`].
pub(super) struct ParameterHeader<'a>(pub(super) &'a Header);

impl ToTokens for ParameterHeader<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.0.name;
        let description = quote_option(&self.0.description);
        let required = &self.0.required;
        let deprecated = &self.0.deprecated;
        let schema = self.0.schema();
        let new_tokens = quote! {
            okapi::openapi3::Parameter {
                name: #name.into(),
                location: "header".into(),
                description: #description,
                required: #required,
                deprecated: #deprecated,
                allow_empty_value: false,
                value: #schema,
                extensions: Default::default(),
            }
        };
        tokens.extend(new_tokens);
    }
}

/// Wrapper, implementing [`ToTokens`] for [`Header`].
pub(super) struct ResponseHeader<'a>(pub(super) &'a Header);

impl ToTokens for ResponseHeader<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let description = quote_option(&self.0.description);
        let required = &self.0.required;
        let deprecated = &self.0.deprecated;
        let schema = self.0.schema();
        tokens.extend(quote! {
            okapi::openapi3::Header {
                description: #description,
                required: #required,
                deprecated: #deprecated,
                allow_empty_value: false,
                value: #schema,
                extensions: Default::default(),
            }
        });
    }
}

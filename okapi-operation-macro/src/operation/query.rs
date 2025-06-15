use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Path;

use crate::{operation::parameters::ParameterStyle, utils::quote_option};

pub(super) static QUERY_ATTRIBUTE_NAME: &str = "query";

/// Query parameter.
#[derive(Debug, FromMeta)]
pub(super) struct Query {
    name: String,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    required: bool,
    #[darling(default)]
    deprecated: bool,
    #[darling(default)]
    style: Option<ParameterStyle>,
    #[darling(default)]
    explode: Option<bool>,
    #[darling(default)]
    allow_empty_value: bool,
    #[darling(default)]
    allow_reserved: bool,
    schema: Path,
    // TODO: support content as well
}

impl ToTokens for Query {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let description = quote_option(&self.description);
        let required = &self.required;
        let deprecated = &self.deprecated;
        let style = quote_option(&self.style);
        let explode = quote_option(&self.explode);
        let allow_empty_values = &self.allow_empty_value;
        let allow_reserved = &self.allow_reserved;
        let ty = &self.schema;
        tokens.extend(quote! {
            okapi::openapi3::Parameter {
                name: #name.into(),
                location: "query".into(),
                description: #description,
                required: #required,
                deprecated: #deprecated,
                allow_empty_value: #allow_empty_values,
                value: {
                    okapi::openapi3::ParameterValue::Schema {
                        style: #style,
                        explode: #explode,
                        allow_reserved: #allow_reserved,
                        schema: components.schema_for::<#ty>(),
                        example: Default::default(),
                        examples: Default::default(),
                    }
                },
                extensions: Default::default(),
            }
        });
    }
}

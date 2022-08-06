use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Path;

use crate::{operation::parameters::ParameterStyle, utils::quote_option};

pub(super) static COOKIE_ATTRIBUTE_NAME: &str = "cookie";

/// Cookie parameter.
#[derive(Debug, FromMeta)]
pub(super) struct Cookie {
    name: String,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    required: bool,
    #[darling(default)]
    deprecated: bool,
    #[darling(default)]
    explode: Option<bool>,
    #[darling(default)]
    allow_empty_value: bool,
    schema: Path,
    // TODO: support content as well
}

impl ToTokens for Cookie {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let description = quote_option(&self.description);
        let required = &self.required;
        let deprecated = &self.deprecated;
        let style = ParameterStyle::Form;
        let explode = quote_option(&self.explode);
        let allow_empty_values = &self.allow_empty_value;
        let allow_reserved = false;
        let ty = &self.schema;
        tokens.extend(quote! {
            okapi::openapi3::Parameter {
                name: #name.into(),
                location: "Cookie".into(),
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

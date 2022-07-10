use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{operation::parameters::ParameterStyle, utils::quote_option};

pub(super) static PATH_ATTRIBUTE_NAME: &str = "path";

/// Path parameter.
#[derive(Debug, FromMeta)]
pub(super) struct Path {
    name: String,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    deprecated: bool,
    #[darling(default)]
    style: Option<ParameterStyle>,
    schema: syn::Path,
    // TODO: support content as well
}

impl ToTokens for Path {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let description = quote_option(&self.description);
        let deprecated = &self.deprecated;
        let style = quote_option(&self.style);
        let ty = &self.schema;
        tokens.extend(quote! {
            okapi::openapi3::Parameter {
                name: #name.into(),
                location: "path".into(),
                description: #description,
                required: true,
                deprecated: #deprecated,
                allow_empty_value: false,
                value: {
                    okapi::openapi3::ParameterValue::Schema {
                        style: #style,
                        explode: None,
                        allow_reserved: false,
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

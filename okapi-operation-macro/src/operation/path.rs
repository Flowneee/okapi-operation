use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

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
    #[darling(default)]
    content: Option<String>,
}

impl ToTokens for Path {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let description = quote_option(&self.description);
        let deprecated = &self.deprecated;
        let ty = &self.schema;
        let value = if let Some(media_type) = &self.content {
            quote! {
                okapi::openapi3::ParameterValue::Content {
                    content: {
                        let mut __map = okapi::Map::new();
                        __map.insert(
                            #media_type.to_string(),
                            okapi::openapi3::MediaType {
                                schema: Some(components.schema_for::<#ty>()),
                                ..Default::default()
                            },
                        );
                        __map
                    },
                }
            }
        } else {
            let style = quote_option(&self.style);
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
        };
        tokens.extend(quote! {
            okapi::openapi3::Parameter {
                name: #name.into(),
                location: "path".into(),
                description: #description,
                required: true,
                deprecated: #deprecated,
                allow_empty_value: false,
                value: { #value },
                extensions: Default::default(),
            }
        });
    }
}

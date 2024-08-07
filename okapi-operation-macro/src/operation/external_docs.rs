use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::utils::quote_option;

#[derive(Debug, FromMeta, PartialEq)]
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
                url: #url.into(),
                description: #description,
                ..Default::default()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Meta};

    use super::*;

    #[test]
    fn parse() {
        let url = "test_url".to_string();
        let description = "test_description".to_string();

        let input: Meta = parse_quote! { external_docs(url = #url, description = #description) };

        assert_eq!(
            ExternalDocs::from_meta(&input).expect("Successfullt parsed"),
            ExternalDocs {
                url,
                description: Some(description)
            }
        );
    }

    // NOTE: codegen test is kinda useless by itself, better to test expectations
    // in some kind of integration tests. But I will leave it here as reference in case
    // they needed.

    // #[test]
    // fn codegen() {
    //     let url = "test_url".to_string();
    //     let description = "test_description".to_string();

    //     let test_value = ExternalDocs {
    //         url: url.clone(),
    //         description: Some(description.clone()),
    //     };

    //     let expected_output = quote! {
    //         okapi::openapi3::ExternalDocs {
    //             url: #url.into(),
    //             description: Some(#description.into()),
    //             ..Default::default()
    //         }
    //     };

    //     assert_eq_tokens::<ExprStruct>(&test_value, &expected_output);
    // }

    // pub fn assert_eq_tokens<T>(actual: &impl ToTokens, expected: &TokenStream)
    // where
    //     T: Parse + Debug + PartialEq,
    // {
    //     let actual: T =
    //         syn::parse2(actual.to_token_stream()).expect("Failed to parse actual value");
    //     let expected: T = syn::parse2(expected.clone()).expect("Failed to parse expected value");

    //     assert_eq!(actual, expected);
    // }
}

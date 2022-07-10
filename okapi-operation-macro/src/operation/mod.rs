use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{AttributeArgs, Ident, ItemFn, Visibility};

use self::{external_docs::ExternalDocs, request_body::RequestBody, response::Responses};
use crate::{
    error::Error,
    operation::{parameters::Parameters, security::Security},
    utils::{attribute_to_args, quote_option, remove_attributes},
    OPENAPI_FUNCTION_NAME_SUFFIX,
};

mod external_docs;
mod header;
mod parameters;
mod path;
mod query;
mod reference;
mod request_body;
mod response;
mod security;

// TODO:
//  - support examples ??
//  - support all fields from Operation/Parameters/Responses
//  - support ToResponses for Result
//  - support for generic functions

static OPENAPI_ATTRIBUTE_NAME: &str = "openapi";

#[derive(Debug, FromMeta)]
struct OperationAttrs {
    #[darling(default)]
    summary: Option<String>,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    operation_id: Option<String>,
    #[darling(default)]
    tags: Option<String>,
    #[darling(default)]
    deprecated: bool,
    #[darling(default)]
    external_docs: Option<ExternalDocs>,
    #[darling(default)]
    parameters: Parameters,
    #[darling(default)]
    responses: Responses,
    #[darling(default)]
    security: Option<Security>,
}

impl ToTokens for OperationAttrs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let summary = quote_option(&self.summary);
        let description = quote_option(&self.description);
        let operation_id = quote_option(&self.operation_id);
        let external_docs = quote_option(&self.external_docs);
        let deprecated = &self.deprecated;
        let tags = {
            let base_str = self.tags.as_deref().unwrap_or_default();
            let values = if !base_str.is_empty() {
                base_str.split(',').map(|y| y.trim()).collect::<Vec<_>>()
            } else {
                vec![]
            };
            quote! {
                vec![
                    #(#values.into()),*
                ]
            }
        };
        let security = quote_option(&self.security);

        let new_tokens = quote! {
            summary: #summary,
            description: #description,
            external_docs: #external_docs,
            operation_id: #operation_id,
            deprecated: #deprecated,
            tags: #tags,
            security: #security,
        };
        tokens.extend(new_tokens);
    }
}

pub(crate) fn openapi(mut attrs: AttributeArgs, mut input: ItemFn) -> Result<TokenStream, Error> {
    for attr in remove_attributes(&mut input.attrs, OPENAPI_ATTRIBUTE_NAME) {
        attrs.extend(attribute_to_args(&attr)?);
    }
    let mut operation_attrs = OperationAttrs::from_list(&attrs)?;
    operation_attrs
        .responses
        .add_return_type(&input, operation_attrs.responses.ignore_return_type);
    let request_body = RequestBody::from_item_fn(&mut input)?;
    let openapi_generator_fn =
        build_openapi_generator_fn(&input.sig.ident, &input.vis, operation_attrs, request_body);
    let output = quote! {
        #input

        #openapi_generator_fn
    };
    Ok(output)
}

fn build_openapi_generator_fn(
    handler_name: &Ident,
    vis: &Visibility,
    attrs: OperationAttrs,
    request_body: Option<RequestBody>,
) -> TokenStream {
    let name = Ident::new(
        &format!("{}{}", handler_name, OPENAPI_FUNCTION_NAME_SUFFIX),
        Span::call_site(),
    );
    let request_body = request_body.map(|x| {
        quote! {
            request_body: Some(okapi::openapi3::RefOr::Object(#x)),
        }
    });
    let parameters = &attrs.parameters;
    let responses = &attrs.responses;
    quote! {
        #vis fn #name(
            components: &mut Components
        ) -> std::result::Result<okapi::openapi3::Operation, anyhow::Error> {
            let mut operation = okapi::openapi3::Operation {
                #attrs
                #request_body
                #responses
                #parameters
                ..Default::default()
            };
            Ok(operation)
        }
    }
}

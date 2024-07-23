use axum::{response::Html, Form, Json};
use mime::{APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED, TEXT_HTML};
use okapi::{
    map,
    openapi3::{MediaType, RefOr, Response, Responses},
    Map,
};

use crate::{
    impl_to_media_types_for_wrapper, impl_to_responses_for_wrapper, Components, ToMediaTypes,
    ToResponses,
};

// Json
impl_to_media_types_for_wrapper!(Json<T>, APPLICATION_JSON.to_string());
impl_to_responses_for_wrapper!(Json<T>);

// Form
impl_to_media_types_for_wrapper!(Form<T>, APPLICATION_WWW_FORM_URLENCODED.to_string());
impl_to_responses_for_wrapper!(Form<T>);

// Html
impl<T> ToMediaTypes for Html<T> {
    fn generate(_components: &mut Components) -> Result<Map<String, MediaType>, anyhow::Error> {
        Ok(map! {
            TEXT_HTML.to_string() => MediaType::default()
        })
    }
}

impl<T> ToResponses for Html<T> {
    fn generate(components: &mut Components) -> Result<Responses, anyhow::Error> {
        Ok(Responses {
            responses: map! {
                "200".into() =>  RefOr::Object(Response {
                    content: <Self as ToMediaTypes>::generate(components)?,
                    ..Default::default()
                }),
            },
            ..Default::default()
        })
    }
}

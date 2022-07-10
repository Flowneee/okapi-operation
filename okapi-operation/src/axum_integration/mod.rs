#![doc = include_str!("../../docs/axum_integration.md")]

pub use concat_idents::concat_idents;

pub use self::{
    handler_traits::{HandlerExt, HandlerWithOperation, ServiceExt, ServiceWithOperation},
    method_router::*,
    router::Router,
};

mod handler_traits;
mod method_router;
mod operations;
mod router;
mod trait_impls;

use axum::{Extension, Json};
use okapi::openapi3::OpenApi;

use crate::*;

/// Serves OpenAPI specification, passed as extension.
#[openapi(
    summary = "OpenAPI specification",
    external_docs(url = "https://swagger.io/specification/"),
    operation_id = "openapi_spec",
    tags = "openapi",
    responses(
        ignore_return_type = true,
        response(
            status = "200",
            description = "",
            content = "axum::Json<std::collections::HashMap<String, String>>"
        )
    )
)]
pub async fn serve_openapi_spec(spec: Extension<OpenApi>) -> Json<OpenApi> {
    Json(spec.0)
}

/// Macro for expanding and binding OpenAPI operation to handler.
#[rustfmt::skip]
#[macro_export]
macro_rules! openapi_handler {
    ($p:path) => {
        $crate::axum_integration::concat_idents!(
            fn_name = $p, __openapi {
                $crate::axum_integration::HandlerExt::with_openapi($p, fn_name,)
            }
        )
    };
}

/// Macro for expanding and binding OpenAPI operation to service.
#[rustfmt::skip]
#[macro_export]
macro_rules! openapi_service {
    ($p:path) => {
        $crate::axum_integration::concat_idents!(
            fn_name = $p, __openapi {
                $crate::axum_integration::ServiceExt::with_openapi($p, fn_name,)
            }
        )
    };
}

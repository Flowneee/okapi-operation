#![doc = include_str!("../../docs/axum_integration.md")]

#[doc(hidden)]
pub use paste::paste;

pub use self::{
    handler_traits::{HandlerExt, HandlerWithOperation, ServiceExt, ServiceWithOperation},
    method_router::*,
    router::{Router, DEFAULT_OPENAPI_PATH},
};

#[cfg(feature = "yaml")]
mod yaml;

mod handler_traits;
mod method_router;
mod operations;
mod router;
mod trait_impls;
mod utils;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use http::{
    header::{self, ACCEPT},
    HeaderMap, HeaderValue, StatusCode,
};
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
pub async fn serve_openapi_spec(spec: State<OpenApi>, headers: HeaderMap) -> Response {
    let accept_header = headers
        .get(ACCEPT)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_ascii_lowercase());

    match accept_header {
        #[cfg(feature = "yaml")]
        Some(accept_header) if accept_header.contains("yaml") => yaml::Yaml(spec.0).into_response(),
        Some(accept_header) if accept_header.contains("json") | accept_header.contains("*/*") => {
            Json(spec.0).into_response()
        }
        Some(_) => {
            let status = StatusCode::BAD_REQUEST;
            let headers = [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )];
            let err = if cfg!(feature = "yaml") {
                "Bad Accept header value, should contain either 'json', 'yaml' or empty"
            } else {
                "Bad Accept header value, should contain either 'json' or empty"
            };
            (status, headers, err).into_response()
        }
        None => {
            // Defaults to json
            Json(spec.0).into_response()
        }
    }
}

/// Macro for expanding and binding OpenAPI operation specification
/// generator to handler or service.
#[rustfmt::skip]
#[macro_export]
macro_rules! openapi_handler {
    // Entry point
    ($($va:ident)::+ $(:: <$($gen_param:tt),+>)?) => {
        $crate::openapi_handler!(@inner $($va)+; ; $($($gen_param)+)?)
    };

    // Each rule have semicolon-separated "arguments"

    // Split input into path and function name, consuming left path segment
    // and pushing it to accumulator.
    //
    // Arguments:
    //   - unprocessed input
    //   - accumulator
    (@inner $va:ident $($vb:ident)+ ; $(:: $acc:ident)*; $($gen_param:tt)*) => {
        $crate::openapi_handler!(@inner $($vb)+; $(:: $acc)* :: $va; $($gen_param)*)
    };
    (@inner $va:ident ; $(:: $acc:ident)*; $($gen_param:tt)*) => {
        $crate::openapi_handler!(@final $va; $($acc)::*; $($gen_param)*)
    };
    
    // Generate code
    //
    // Arguments:
    //   - function name
    //   - path to function
    (@final $fn_name:ident ; $($prefix_path_part:ident)::* ; $($gen_param:tt)*) => {
        $crate::axum_integration::paste!{
            {
                #[allow(unused_imports)]
                use $crate::axum_integration::{HandlerExt, ServiceExt};

                $($prefix_path_part ::)* $fn_name :: <$($gen_param),*>
                    .with_openapi($($prefix_path_part ::)* [<$fn_name __openapi>])
            }
        }
    };
}

/// Macro for expanding and binding OpenAPI operation specification
/// generator to handler or service (shortcut to [`openapi_handler`])
#[rustfmt::skip]
#[macro_export]
macro_rules! oh {
    ($($v:tt)+) => {
        $crate::openapi_handler!($($v)+)
    };

}

/// Macro for expanding and binding OpenAPI operation specification
/// generator to handler or service.
#[rustfmt::skip]
#[macro_export]
#[deprecated = "Use `openapi_handler` instead"]
macro_rules! openapi_service {
    ($($t:tt)+) => {
        {
            $crate::openapi_handler!($($t)+)
        }
    }
}

// tests in tests/axum_integration.rs because of https://github.com/rust-lang/rust/issues/52234

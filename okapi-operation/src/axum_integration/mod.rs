#![doc = include_str!("../../docs/axum_integration.md")]

#[doc(hidden)]
pub use paste::paste;

pub use self::{
    handler_traits::{HandlerExt, HandlerWithOperation, ServiceExt, ServiceWithOperation},
    method_router::*,
    router::{Router, DEFAULT_OPENAPI_PATH},
};

// #[cfg(feature = "macro")]
// #[doc(inline)]
// pub use okapi_operation_macro::openapi_handler;

#[cfg(feature = "yaml")]
mod yaml;

mod handler_traits;
mod method_router;
mod operations;
mod router;
mod trait_impls;
mod utils;

use axum::response::{IntoResponse, Response};
use axum::{extract::State, Json};
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

//  $(:: <$($gen_param:tt),+>)?

/// Macro for expanding and binding OpenAPI operation specification
/// generator to handler or service.
#[rustfmt::skip]
#[macro_export]
macro_rules! openapi_handler {
    // Just name
    ($fn_name:ident) => {
        openapi_handler!(@final $fn_name)
    };

    // Path
    ($va:ident $(:: $vb:ident)*) => {
        openapi_handler!(@path $va $($vb)*)
    };
    (@path $va:ident $($vb:ident)+) => {
        openapi_handler!(@path $($vb)+; $va)
    };
    (@path $va:ident $($vb:ident)+; $($acc:ident)::+) => {
        openapi_handler!(@path $($vb)+; $($acc)::+ :: $va)
    };
    (@path $va:ident; $($acc:ident)::+) => {
        openapi_handler!(@final $va, $($acc)::+)
    };

    (@final $fn_name:ident $(, $($prefix_path_part:ident)::*)?) => {
        $crate::axum_integration::paste!{
            {
                #[allow(unused_imports)]
                use $crate::axum_integration::{HandlerExt, ServiceExt};

                $($($prefix_path_part)::* ::)? $fn_name
                    .with_openapi($($($prefix_path_part)::* ::)? [<$fn_name __openapi>])
            }
        }
    };
}

/// Macro for expanding and binding OpenAPI operation specification
/// generator to handler or service.
#[rustfmt::skip]
#[macro_export]
#[deprecated = "Use `openapi_handler` instead"]
macro_rules! openapi_service {
    ($($t:tt)*) => {
        {
            openapi_handler!($($t)*)
        }
    }
}

#[cfg(test)]
mod openapi_macro {
    use axum::body::Body;
    use http::Request;

    use super::*;

    #[test]
    fn openapi_handler_name() {
        #[openapi]
        async fn handle() {}

        let _ = Router::<()>::new().route("/", get(openapi_handler!(handle)));
    }

    #[test]
    fn openapi_handler_path() {
        mod outer {
            pub mod inner {
                use crate::*;

                #[openapi]
                pub async fn handle() {}
            }
        }

        let _ = Router::<()>::new().route("/", get(openapi_handler!(outer::inner::handle)));
    }

    #[test]
    fn openapi_handler_method() {
        struct S {}

        impl S {
            #[openapi]
            async fn handle() {}
        }

        let _ = Router::<()>::new().route("/", get(openapi_handler!(S::handle)));
    }

    // #[test]
    // fn openapi_handler_typed() {
    //     #[openapi]
    //     async fn handle<T>() {}

    //     let _ = Router::<()>::new().route("/", get(openapi_handler!(handle::<()>)));
    // }

    // #[test]
    // fn openapi_handler_method_typed() {
    //     struct S<T>(T);

    //     impl<T> S<T> {
    //         #[openapi]
    //         async fn handle<U>() {}
    //     }

    //     let _ = Router::<()>::new().route("/", get(openapi_handler!(S::<()>::handle::<()>)));
    // }

    #[test]
    #[allow(deprecated)]
    fn openapi_service_name() {
        #[openapi]
        async fn service(_: Request<Body>) {
            unimplemented!()
        }

        let _ = Router::<()>::new().route("/", get(openapi_service!(service)));
    }
}

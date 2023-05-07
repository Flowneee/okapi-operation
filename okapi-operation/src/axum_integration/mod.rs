#![doc = include_str!("../../docs/axum_integration.md")]

#[doc(hidden)]
pub use paste::paste;

pub use self::{
    handler_traits::{HandlerExt, HandlerWithOperation, ServiceExt, ServiceWithOperation},
    method_router::*,
    router::Router,
};

#[cfg(feature = "axum-yaml")]
mod yaml;

mod handler_traits;
mod method_router;
mod operations;
mod router;
mod trait_impls;

use axum::response::{IntoResponse, Response};
use axum::{extract::State, Json};
use http::HeaderMap;
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
    match 1 {
        #[cfg(feature = "axum-yaml")]
        1 => yaml::axum_yaml_serve_spec(spec, headers).await,
        _ => default_serve_spec(spec, headers).await,
    }
}

async fn default_serve_spec(spec: State<OpenApi>, _headers: HeaderMap) -> Response {
    Json(spec.0).into_response()
}

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
    ($va:ident $(:: $vb:ident)+) => {
        openapi_handler!(@path $va $($vb)+)
    };
    (@path $va:ident $($vb:ident)+) => {
        openapi_handler!(@path $($vb)+; $va)
    };
    (@path $va:ident $($vb:ident)+; $acc_a:ident $(:: $acc_b:ident)*) => {
        openapi_handler!(@path $($vb)+; $acc_a $(:: $acc_b)* :: $va)
    };
    (@path $va:ident; $acc_a:ident $(:: $acc_b:ident)*) => {
        openapi_handler!(@final $va, $acc_a $(:: $acc_b)*)
    };

    (@final $fn_name:ident $(, $prefix_a:ident $(:: $prefix_b:ident)*)?) => {
        $crate::axum_integration::paste!{
            {
                #[allow(unused_imports)]
                use $crate::axum_integration::{HandlerExt, ServiceExt};

                $($prefix_a $(:: $prefix_b)* ::)? $fn_name
                    .with_openapi($($prefix_a $(:: $prefix_b)* ::)? [<$fn_name __openapi>])
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
            use $crate::*;
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

        let _ = Router::<(), Body>::new().route("/", get(openapi_handler!(handle)));
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

        let _ = Router::<(), Body>::new().route("/", get(openapi_handler!(outer::inner::handle)));
    }

    #[test]
    fn openapi_service_name() {
        #[openapi]
        async fn service(_: Request<Body>) {
            unimplemented!()
        }

        let _ = Router::<(), Body>::new().route("/", get(openapi_handler!(service)));
    }

    #[test]
    fn openapi_service_path() {
        mod outer {
            pub mod inner {
                use axum::body::Body;
                use http::Request;

                use crate::*;

                #[openapi]
                pub async fn service(_: Request<Body>) {
                    unimplemented!()
                }
            }
        }

        let _ = Router::<(), Body>::new().route("/", get(openapi_handler!(outer::inner::service)));
    }

    #[allow(deprecated)]
    #[test]
    fn openapi_service_alias_name() {
        #[openapi]
        async fn service(_: Request<Body>) {
            unimplemented!()
        }

        let _ = Router::<(), Body>::new().route("/", get(openapi_service!(service)));
    }

    #[allow(deprecated)]
    #[test]
    fn openapi_service_alias_path() {
        mod outer {
            pub mod inner {
                use axum::body::Body;
                use http::Request;

                use crate::*;

                #[openapi]
                pub async fn service(_: Request<Body>) {
                    unimplemented!()
                }
            }
        }

        let _ = Router::<(), Body>::new().route("/", get(openapi_service!(outer::inner::service)));
    }
}

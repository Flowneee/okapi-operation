#[cfg(feature = "axum-integration")]
mod tests {
    use axum::body::Body;
    use http::Request;
    use okapi_operation::{
        axum_integration::{get, Router},
        oh, openapi, openapi_handler, openapi_service, Components, ToResponses,
    };

    #[test]
    fn openapi_handler_name() {
        #[openapi]
        async fn handle() {}

        let _ = Router::<()>::new().route("/", get(oh!(handle)));
    }

    #[test]
    fn openapi_handler_path() {
        mod outer {
            pub mod inner {
                use okapi_operation::*;

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

    #[test]
    fn openapi_handler_typed() {
        #[openapi]
        async fn handle<T>() {}

        let _ = Router::<()>::new().route("/", get(openapi_handler!(handle::<()>)));
    }

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

#[cfg(feature = "axum")]
#[allow(deprecated)]
mod openapi {
    use axum::Json;
    use okapi::{openapi3::RefOr, schemars::gen::SchemaGenerator};
    use okapi_operation::{
        axum_integration::{get, Router},
        oh, openapi, Components, ToMediaTypes, ToResponses,
    };

    #[test]
    fn json_body_detection() {
        #[openapi]
        async fn handle(_arg: Json<String>) {}

        let schema = Router::<()>::new()
            .route("/", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("Schema generation shoildn't fail");

        let operation = schema.paths["/"]
            .clone()
            .get
            .expect("GET / should be present")
            .request_body
            .expect("GET / request body should be present");
        let RefOr::Object(request_body) = operation else {
            panic!("GET / request body should be RefOr::Object");
        };

        let body_schema = request_body.content["application/json"]
            .clone()
            .schema
            .expect("GET / request body schema should be present");

        let mut gen = SchemaGenerator::default();
        let expected_schema = gen.subschema_for::<String>().into_object();

        assert_eq!(body_schema, expected_schema);
    }
}

#[cfg(feature = "axum")]
#[allow(deprecated)]
mod openapi_handler {
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

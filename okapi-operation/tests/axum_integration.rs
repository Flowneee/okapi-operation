#[cfg(feature = "axum")]
#[allow(deprecated)]
mod openapi {
    use axum::Json;
    use okapi::{openapi3::RefOr, schemars::gen::SchemaGenerator};
    use okapi_operation::{
        axum_integration::{Router, get},
        oh, openapi,
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

    #[test]
    fn string_body_detection() {
        #[openapi]
        async fn handle(_arg: String) {}

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

        assert!(
            request_body.content["text/plain"].clone().schema.is_none(),
            "String body (text/plain) shouldn't have schema"
        );
    }
}

#[cfg(feature = "axum")]
mod parameters {
    use okapi::openapi3::{ParameterValue, RefOr};
    use okapi_operation::{
        axum_integration::{Router, get},
        oh, openapi,
    };

    fn get_parameter(name: &str) -> okapi::openapi3::Parameter {
        #[openapi(parameters(
            query(name = "schema-param", required = true, schema = "String"),
            query(
                name = "content-param",
                required = false,
                content = "application/json",
                schema = "String"
            ),
            path(name = "path-param", content = "text/plain", schema = "u32"),
            header(name = "x-custom", schema = "String"),
            header(
                name = "x-custom-content",
                content = "application/json",
                schema = "String"
            ),
        ))]
        async fn handle() {}

        let schema = Router::<()>::new()
            .route("/", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let operation = schema.paths["/"]
            .clone()
            .get
            .expect("GET / should be present");
        operation
            .parameters
            .into_iter()
            .find_map(|p| match p {
                RefOr::Object(p) if p.name == name => Some(p),
                _ => None,
            })
            .unwrap_or_else(|| panic!("parameter '{}' not found", name))
    }

    #[test]
    fn query_schema_value() {
        let param = get_parameter("schema-param");
        assert!(
            matches!(param.value, ParameterValue::Schema { .. }),
            "query with schema= should produce ParameterValue::Schema"
        );
    }

    #[test]
    fn query_content_value() {
        let param = get_parameter("content-param");
        let ParameterValue::Content { content } = param.value else {
            panic!("query with content= should produce ParameterValue::Content");
        };
        assert!(
            content.contains_key("application/json"),
            "content map should contain 'application/json'"
        );
        assert!(
            content["application/json"].schema.is_some(),
            "MediaType schema should be present"
        );
    }

    #[test]
    fn path_content_value() {
        let param = get_parameter("path-param");
        let ParameterValue::Content { content } = param.value else {
            panic!("path with content= should produce ParameterValue::Content");
        };
        assert!(content.contains_key("text/plain"));
    }

    #[test]
    fn header_schema_value() {
        let param = get_parameter("x-custom");
        assert!(matches!(param.value, ParameterValue::Schema { .. }));
    }

    #[test]
    fn header_content_value() {
        let param = get_parameter("x-custom-content");
        let ParameterValue::Content { content } = param.value else {
            panic!("header with content= should produce ParameterValue::Content");
        };
        assert!(content.contains_key("application/json"));
    }
}

#[cfg(feature = "axum")]
#[allow(deprecated)]
mod openapi_handler {
    use axum::body::Body;
    use http::Request;
    use okapi_operation::{
        axum_integration::{Router, get},
        oh, openapi, openapi_handler, openapi_service,
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
        #[allow(clippy::extra_unused_type_parameters)]
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

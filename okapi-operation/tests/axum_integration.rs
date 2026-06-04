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
            .route("/{path-param}", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let operation = schema.paths["/{path-param}"]
            .clone()
            .get
            .expect("GET /{path-param} should be present");
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

#[cfg(feature = "axum")]
mod path_inference {
    use axum::extract::Path;
    use okapi::openapi3::{ParameterValue, RefOr};
    use okapi_operation::{
        JsonSchema,
        axum_integration::{Router, get, post},
        // `schemars` must be in scope: the `JsonSchema` derive expands to `schemars::` paths.
        oh,
        openapi,
        schemars,
    };
    use serde::Deserialize;

    fn parameters(
        route: &str,
        method: &str,
        op: okapi::openapi3::Operation,
    ) -> Vec<okapi::openapi3::Parameter> {
        let _ = (route, method);
        op.parameters
            .into_iter()
            .map(|p| match p {
                RefOr::Object(obj) => obj,
                RefOr::Ref(_) => panic!("unexpected ref parameter"),
            })
            .collect()
    }

    fn get_op(schema: &okapi::openapi3::OpenApi, route: &str) -> okapi::openapi3::Operation {
        schema.paths[route]
            .clone()
            .get
            .expect("GET should be present")
    }

    fn post_op(schema: &okapi::openapi3::OpenApi, route: &str) -> okapi::openapi3::Operation {
        schema.paths[route]
            .clone()
            .post
            .expect("POST should be present")
    }

    fn assert_path_param(p: &okapi::openapi3::Parameter, name: &str) {
        assert_eq!(p.name, name, "param name mismatch");
        assert_eq!(p.location, "path", "param location mismatch");
        assert!(p.required, "path params must be required");
        assert!(
            matches!(p.value, ParameterValue::Schema { .. }),
            "expected schema parameter value"
        );
    }

    #[test]
    fn infers_single_path_parameter() {
        #[openapi]
        async fn handle(Path(system): Path<String>) {
            let _ = system;
        }

        let schema = Router::<()>::new()
            .route("/api/{system}", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let params = parameters("/api/{system}", "GET", get_op(&schema, "/api/{system}"));
        assert_eq!(params.len(), 1);
        assert_path_param(&params[0], "system");
    }

    #[test]
    fn infers_tuple_path_parameters_in_order() {
        #[openapi]
        async fn abort_backup(Path((system, backup_name)): Path<(String, String)>) {
            let _ = (system, backup_name);
        }

        let schema = Router::<()>::new()
            .route(
                "/api/system/{system}/backup/abort/{backup_name}",
                post(oh!(abort_backup)),
            )
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let params = parameters(
            "/api/system/{system}/backup/abort/{backup_name}",
            "POST",
            post_op(&schema, "/api/system/{system}/backup/abort/{backup_name}"),
        );
        assert_eq!(params.len(), 2, "two path params expected");
        assert_path_param(&params[0], "system");
        assert_path_param(&params[1], "backup_name");
    }

    #[test]
    fn explicit_declaration_wins_over_inferred() {
        // Explicit `description` should survive — inference must not overwrite
        // a parameter already declared by name.
        #[openapi(parameters(path(
            name = "system",
            description = "system id",
            schema = "String"
        )))]
        async fn handle(Path(system): Path<String>) {
            let _ = system;
        }

        let schema = Router::<()>::new()
            .route("/api/{system}", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let params = parameters("/api/{system}", "GET", get_op(&schema, "/api/{system}"));
        assert_eq!(params.len(), 1, "no duplicate from inference");
        assert_eq!(params[0].name, "system");
        assert_eq!(params[0].description.as_deref(), Some("system id"));
    }

    #[test]
    fn unsupported_patterns_are_skipped() {
        // Wildcard binding cannot be inferred — the user is expected to declare
        // the parameter explicitly. We just verify the macro still compiles
        // and produces an operation with no inferred params.
        #[openapi]
        async fn handle(Path(_): Path<String>) {}

        let schema = Router::<()>::new()
            .route("/api/{system}", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let op = get_op(&schema, "/api/{system}");
        assert!(op.parameters.is_empty(), "no params should be inferred");
    }

    #[test]
    fn infers_struct_path_parameters_per_field() {
        #[derive(Deserialize, JsonSchema)]
        struct Params {
            system: String,
            backup_id: u32,
        }

        #[openapi]
        async fn handle(Path(params): Path<Params>) {
            let _ = params;
        }

        let schema = Router::<()>::new()
            .route("/api/{system}/backup/{backup_id}", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let params = parameters(
            "/api/{system}/backup/{backup_id}",
            "GET",
            get_op(&schema, "/api/{system}/backup/{backup_id}"),
        );
        assert_eq!(params.len(), 2, "one param per struct field");
        // Field declaration order is preserved.
        assert_path_param(&params[0], "system");
        assert_path_param(&params[1], "backup_id");
    }

    #[test]
    fn struct_field_not_in_route_is_rejected() {
        #[derive(Deserialize, JsonSchema)]
        struct Params {
            system: String,
            // `unexpected` has no matching `{placeholder}` in the route below.
            unexpected: String,
        }

        #[openapi]
        async fn handle(Path(params): Path<Params>) {
            let _ = params;
        }

        let result = Router::<()>::new()
            .route("/api/{system}", get(oh!(handle)))
            .generate_openapi_builder()
            .build();

        let err = result.expect_err("mismatched struct field must fail build");
        assert!(
            format!("{err:#}").contains("unexpected"),
            "error should name the offending parameter: {err:#}"
        );
    }

    #[test]
    fn explicit_declaration_wins_over_inferred_struct_field() {
        #[derive(Deserialize, JsonSchema)]
        struct Params {
            system: String,
        }

        #[openapi(parameters(path(
            name = "system",
            description = "system id",
            schema = "String"
        )))]
        async fn handle(Path(params): Path<Params>) {
            let _ = params;
        }

        let schema = Router::<()>::new()
            .route("/api/{system}", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let params = parameters("/api/{system}", "GET", get_op(&schema, "/api/{system}"));
        assert_eq!(params.len(), 1, "no duplicate from inference");
        assert_eq!(params[0].name, "system");
        assert_eq!(params[0].description.as_deref(), Some("system id"));
    }
}

#[cfg(feature = "axum")]
mod cookie_parameters {
    use okapi::openapi3::{ParameterValue, RefOr};
    use okapi_operation::{
        axum_integration::{Router, get},
        oh, openapi,
    };

    fn get_parameters(
        schema: &okapi::openapi3::OpenApi,
        route: &str,
    ) -> Vec<okapi::openapi3::Parameter> {
        schema.paths[route]
            .clone()
            .get
            .expect("GET should be present")
            .parameters
            .into_iter()
            .map(|p| match p {
                RefOr::Object(obj) => obj,
                RefOr::Ref(_) => panic!("unexpected ref parameter"),
            })
            .collect()
    }

    #[test]
    fn cookie_parameter_is_emitted() {
        #[openapi(parameters(cookie(name = "session", required = true, schema = "String")))]
        async fn handle() {}

        let schema = Router::<()>::new()
            .route("/", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let params = get_parameters(&schema, "/");
        assert_eq!(params.len(), 1, "cookie parameter must be emitted");
        let p = &params[0];
        assert_eq!(p.name, "session");
        assert_eq!(p.location, "cookie", "OpenAPI requires lowercase 'cookie'");
        assert!(p.required);
        assert!(matches!(p.value, ParameterValue::Schema { .. }));
    }

    #[test]
    fn cookie_mixed_with_other_parameter_kinds() {
        // Verify cookie params don't collide with header/path/query when
        // declared together on the same operation.
        #[openapi(parameters(
            header(name = "x-trace", schema = "String"),
            query(name = "limit", schema = "u32"),
            cookie(name = "session", schema = "String"),
        ))]
        async fn handle() {}

        let schema = Router::<()>::new()
            .route("/", get(oh!(handle)))
            .generate_openapi_builder()
            .build()
            .expect("schema generation shouldn't fail");

        let params = get_parameters(&schema, "/");
        assert_eq!(params.len(), 3);
        let locations: Vec<&str> = params.iter().map(|p| p.location.as_str()).collect();
        assert!(locations.contains(&"header"));
        assert!(locations.contains(&"query"));
        assert!(locations.contains(&"cookie"));
    }
}

# Integration with axum

- [`Integration with axum`](#-integration-with-axum-)
  - [Example](#example)
  - [Customizing `OpenApiBuilder`](#customizing-openapibuilder)
  - [Detecting request body and parameters from arguments](#detecting-request-body-and-parameters-from-arguments)

This module provide integration with [`axum`] based on `#[openapi]` macro.

Integration is done by replacing [`axum::Router`] and [`axum::routing::MethodRouter`] with drop-in replacements, which support binding OpenAPI operation to handler using [`openapi_handler`] and [`openapi_service`] macros.

## Example

This is example from root of this crate, but this time with [`axum_integration`].

```no_run
use axum::{extract::Query, Json};
use okapi_operation::{axum_integration::*, *};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct Request {
    /// Echo data
    data: String,
}

#[openapi(
    summary = "Echo using GET request",
    operation_id = "echo_get",
    tags = "echo",
    parameters(
        query(name = "echo-data", required = true, schema = "std::string::String",),
        header(name = "x-request-id", schema = "std::string::String",)
    )
)]
async fn echo_get(query: Query<Request>) -> Json<String> {
    Json(query.0.data)
}

#[openapi(
    summary = "Echo using POST request",
    operation_id = "echo_post",
    tags = "echo"
)]
async fn echo_post(
    #[body(description = "Echo data", required = true)] body: Json<Request>,
) -> Json<String> {
    Json(body.0.data)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/echo/get", get(openapi_handler!(echo_get)))
        .route("/echo/post", post(openapi_handler!(echo_post)))
        .finish_openapi("/openapi", "Demo", "1.0.0")
        .expect("no problem");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap()
}
```

## Customizing [`OpenApiBuilder`]

By default [`Router`] have an empty [`OpenApiBuilder::default()`] inside, which is used as template when generating specification. The only 2 mandatory fields in specification is set when calling [`Router::finish_openapi`].

If you need to customize builder template, you can either:

- access existing builder with [`Router::openapi_builder_template_mut`] (example below) or [`Router::update_openapi_builder_template`];
- prepare your own builder and set it with [`Router::set_openapi_builder_template`].

```no_run
use axum::{extract::Query, Json};
use okapi_operation::{axum_integration::*, *};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let mut app = Router::new();
        
    // Setting description and ToS.
    app.openapi_builder_template_mut()
        .description("Some description")
        .terms_of_service("Terms of Service");
        
    // Proceed as usual.
    let app = app
        .finish_openapi("/openapi", "Demo", "1.0.0")
        .expect("no problem");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap()
}
```

## Detecting request body and parameters from arguments

Request body and some parameters can be automatically detected from function arguments without explicitly marking or describing them. Detection is done simply by type name, i.e. JSON body will be detected from `Json`, `axum::Json`, `reexported::axum::Json`, etc.

Supported request bodies:

- [`String`] (as `text/plain`)
- [`axum::extract::Json`]
- [`bytes::Bytes`] (as `application/octet_stream`)

# `okapi-operation`

[![Crates.io](https://img.shields.io/crates/v/okapi-operation)](https://crates.io/crates/okapi-operation)
[![docs.rs](https://img.shields.io/docsrs/okapi-operation/latest)](https://docs.rs/okapi-operation/latest)
![CI](https://github.com/Flowneee/okapi-operation/actions/workflows/ci.yml/badge.svg)

Library which allow to generate OpenAPI's operation definitions (using types from `okapi` crate) with procedural
macro `#[openapi]`.

## Example (with axum-integration feature).

```rust,compile
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
    #[request_body(description = "Echo data", required = true)] body: Json<Request>,
) -> Json<String> {
    Json(body.0.data)
}

fn main() {
    // Here you can also add security schemes, other operations, modify internal OpenApi object.
    let oas_builder = OpenApiBuilder::new("Demo", "1.0.0");
    
    let app = Router::new()
        .route("/echo/get", get(openapi_handler!(echo_get)))
        .route("/echo/post", post(openapi_handler!(echo_post)))
        .route_openapi_specification("/openapi", oas_builder)
        .expect("no problem");

    let fut = async {
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    };
    //tokio::runtime::Runtime::new().block_on(fut);
}
```

## Features

* `macro`: enables re-import of `#[openapi]` macro (enabled by default);
* `axum-integration`: enables integration with `axum`(https://github.com/tokio-rs/axum) crate (implement traits for
  certain `axum` types):
    * Compatibility with `axum`: since integration heavely rely on `axum` types, this crate will be compatible only with
      few (maybe even one) last versions of `axum`;
    * Currently supported `axum` versions: `0.6.x`.
* `axum-yaml`: enables ability to serve the spec in yaml format in case of present `Accept` header with `yaml` value.
  Otherwise, in case of values `json|*/*` or empty, `json`'s being served.

## TODO

* [ ] support examples on MediaType or Parameter (examples supported on types via `JsonSchema` macro)
* [ ] support inferring schemas of parameters from function definitions
* [ ] support for renaming or changing paths to okapi/schemars/okapi-operations in macro
* [ ] more examples
* [ ] ...

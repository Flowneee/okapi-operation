# `speka` - OpenAPI generation utilities

[![Crates.io](https://img.shields.io/crates/v/speka)](https://crates.io/crates/speka)
[![docs.rs](https://img.shields.io/docsrs/speka/latest)](https://docs.rs/speka/latest)
![CI](https://github.com/Flowneee/speka/actions/workflows/ci.yml/badge.svg)

Library which allow to generate OpenAPI's operation definitions (using types from `okapi` crate) with procedural
macro `#[openapi]` (formerly named `okapi-operation`, see https://github.com/Flowneee/speka/issues/17).

## Example (with axum-integration feature).

```rust,no_run
use axum::{extract::Query, Json};
use speka::{axum_integration::*, *};
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

fn main() {
    let app = Router::new()
        .route("/echo/get", get(openapi_handler!(echo_get)))
        .route("/echo/post", post(openapi_handler!(echo_post)))
        .finish_openapi("/openapi", "Demo", "1.0.0")
        .expect("no problem");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap()
}
```

## Features

* `macro`: enables re-import of `#[openapi]` macro (enabled by default);
* `axum-integration`: enables integration with `axum`(https://github.com/tokio-rs/axum) crate (implement traits for
  certain `axum` types):
    * Compatibility with `axum`: since integration heavely rely on `axum` types, this crate will be compatible only with
      few (maybe even one) last versions of `axum`;
    * Currently supported `axum` versions: `0.7.x`.
* `yaml`: enables ability to serve the spec in yaml format in case of present `Accept` header with `yaml` value.
  Otherwise, in case of values `json|*/*` or empty, `json`'s being served (currently affects only `axum-integration`).

## TODO

* [ ] support examples on MediaType or Parameter (examples supported on types via `JsonSchema` macro)
* [ ] support inferring schemas of parameters from function definitions
* [ ] support for renaming or changing paths to okapi/schemars/spekas in macro
* [ ] more examples
* [ ] ...

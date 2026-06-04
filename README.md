# `okapi-operation`

[![Crates.io](https://img.shields.io/crates/v/okapi-operation)](https://crates.io/crates/okapi-operation)
[![docs.rs](https://img.shields.io/docsrs/okapi-operation/latest)](https://docs.rs/okapi-operation/latest)
![CI](https://github.com/Flowneee/okapi-operation/actions/workflows/ci.yml/badge.svg)

Library which allow to generate OpenAPI's operation definitions (using types from `okapi` crate) with procedural
macro `#[openapi]`.

## Example (with axum-integration feature).

```rust,no_run
use axum::{Json, extract::Query};
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

fn main() {
    let app = Router::new()
        .route("/echo/get", get(openapi_handler!(echo_get)))
        .route("/echo/post", post(openapi_handler!(echo_post)))
        .finish_openapi("/openapi", "Demo", "1.0.0")
        .expect("no problem");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}
```

## Path parameters from function signature (axum)

With the `axum` feature enabled, path parameters are inferred from the
function signature, so handlers that use the axum `Path<...>` extractor no
longer need to repeat their names in `parameters(path(...))`. Three binding
shapes are recognized:

```rust,no_run
use axum::extract::Path;
use okapi_operation::{axum_integration::*, *};
use serde::Deserialize;

// Single path parameter — inferred as `system: String`.
#[openapi(operation_id = "get_system")]
async fn get_system(Path(system): Path<String>) -> String {
    system
}

// Tuple — inferred in order: `system: String`, `backup_name: String`.
#[openapi(operation_id = "abort_backup")]
async fn abort_backup(Path((system, backup_name)): Path<(String, String)>) {
    let _ = (system, backup_name);
}

// Struct — inferred as one parameter per field (matching how axum
// deserializes `Path<Struct>` by field name): `system: String`, `backup_id: u32`.
#[derive(Deserialize, JsonSchema)]
struct BackupParams {
    system: String,
    backup_id: u32,
}

#[openapi(operation_id = "get_backup")]
async fn get_backup(Path(params): Path<BackupParams>) {
    let _ = params;
}
```

Struct fields are matched to the route by name, so the type must implement
`JsonSchema`. Wildcard `_` bindings and reference parameters are left to
explicit `parameters(path(...))` declarations, which always take precedence
over inferred entries with the same name.

When the specification is built, every declared/inferred `path` parameter is
validated against the route template: building fails if a parameter has no
matching `{placeholder}` in the route (e.g. a misspelled struct field). Extra
placeholders in the route are allowed.

## Cookie parameters

Cookie parameters are declared the same way as the other kinds:

```rust,no_run
use okapi_operation::*;

#[openapi(
    operation_id = "me",
    parameters(cookie(name = "session", required = true, schema = "String")),
)]
async fn me() {}
## Parameters

Parameters (`path`, `query`, `header`, `cookie`) support two mutually exclusive ways to describe the parameter value:

- `schema = "Type"` — describes the parameter inline using a JSON Schema derived from the given type (default, as in the example above);
- `content = "media/type", schema = "Type"` — describes the parameter via a media type map (`ParameterValue::Content`), useful when the parameter encoding requires a specific media type (e.g. `application/json`).

```rust,no_run
#[openapi(
    parameters(
        // inline schema
        query(name = "id", schema = "u64"),
        // content-based
        query(name = "filter", content = "application/json", schema = "MyFilter"),
        path(name = "slug", content = "text/plain", schema = "String"),
    )
)]
async fn handler() { ... }
```

## Features

- `macro`: enables re-import of `#[openapi]` macro (enabled by default);
- `axum-integration`: enables integration with `axum`(https://github.com/tokio-rs/axum) crate (implement traits for
  certain `axum` types):
  - Compatibility with `axum`: since integration heavely rely on `axum` types, this crate will be compatible only with
    few (maybe even one) last versions of `axum`;
  - Currently supported `axum` versions: `0.7.x`.
- `yaml`: enables ability to serve the spec in yaml format in case of present `Accept` header with `yaml` value.
  Otherwise, in case of values `json|*/*` or empty, `json`'s being served (currently affects only `axum-integration`).

## TODO

- [ ] support examples on MediaType or Parameter (examples supported on types via `JsonSchema` macro)
- [ ] support inferring schemas of parameters from function definitions
- [ ] support for renaming or changing paths to okapi/schemars/okapi-operations in macro
- [ ] more examples
- [ ] introduce MSRV policy
- [ ] ...

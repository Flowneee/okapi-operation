# `okapi-operation`

Library which allow to generate OpenAPI's operation definitions (using types from [`okapi`] crate) with procedural macro [`openapi`].

## Example (with axum)

```rust,ignore
use axum::{
    extract::Query,
    http::Method,
    routing::{get, post},
    Json, Router,
};
use okapi_operation::*;
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

async fn openapi_spec() -> Json<OpenApi> {
    let generate_spec = || {
        OpenApiBuilder::new("Echo API", "1.0.0")
            .add_operation("/echo/get", Method::GET, echo_get__openapi)?
            .add_operation("/echo/post", Method::POST, echo_post__openapi)?
            .generate_spec()
    };
    generate_spec().map(Json).expect("Should not fail")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/echo/get", get(echo_get))
        .route("/echo/post", post(echo_post))
        .route("/openapi", get(openapi_spec));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Features

* `macro`: enables re-import of [`openapi`] macro (enabled by default);
* `axum-integration`: enables integration with [`axum`](https://github.com/tokio-rs/axum) crate (implement traits for certain `axum` types).

## TODO

* [ ] support cookies
* [ ] support examples
* [ ] support inferring schemas of parameters from function definitions
* [ ] ...

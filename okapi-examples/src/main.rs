use axum::{Json, extract::Query};
use okapi_operation::{axum_integration::*, *};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct Request {
    /// Echo data
    data: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchFilter {
    /// Minimum value
    min: Option<i32>,
    /// Maximum value
    max: Option<i32>,
}

#[openapi(
    summary = "Echo using GET request",
    tags = "echo",
    parameters(
        query(name = "echo-data", required = true, schema = "std::string::String",),
        header(name = "x-request-id", schema = "std::string::String",),
        header(name = "Accept", schema = "std::string::String")
    )
)]
async fn echo_get(query: Query<Request>) -> Json<String> {
    Json(query.0.data)
}

#[openapi(summary = "Echo using POST request", tags = "echo")]
async fn echo_post(
    #[body(description = "Echo data", required = true)] body: Json<Request>,
) -> Json<String> {
    Json(body.0.data)
}

// Detect schema from known types, Json in this case
#[openapi(
    summary = "Echo using PUT request",
    operation_id = "echo_put",
    tags = "echo"
)]
async fn echo_put(body: Json<Request>) -> Json<String> {
    Json(body.0.data)
}

// Demonstrates content-based parameter: the `filter` query parameter is described
// as application/json rather than a plain schema, which is useful for complex objects
// that are JSON-encoded in the query string.
#[openapi(
    summary = "Search with JSON-encoded filter",
    tags = "search",
    parameters(query(
        name = "filter",
        required = false,
        content = "application/json",
        schema = "SearchFilter"
    ),)
)]
async fn search(query: Query<SearchFilter>) -> Json<String> {
    Json(format!("min={:?} max={:?}", query.0.min, query.0.max))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/echo",
            get(oh!(echo_get)).post(oh!(echo_post)).put(oh!(echo_put)),
        )
        .route("/search", get(oh!(search)))
        .finish_openapi("/openapi", "Demo", "1.0.0")
        .expect("no problem");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}

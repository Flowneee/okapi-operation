use axum::{
    extract::Query,
    http::Request,
    middleware::{self, Next},
    response::Response,
    Json,
};
use okapi_operation::{axum_integration::*, *};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct RequestBody {
    /// Echo data
    data: String,
}

#[openapi(
    summary = "Echo using GET request",
    operation_id = "echo_get",
    tags = "echo",
    parameters(
        query(name = "echo-data", required = true, schema = "std::string::String",),
        header(name = "x-request-id", schema = "std::string::String",),
        header(name = "Accept", schema = "std::string::String")
    )
)]
async fn echo_get(query: Query<RequestBody>) -> Json<String> {
    Json(query.0.data)
}

#[openapi(
    summary = "Echo using POST request",
    operation_id = "echo_post",
    tags = "echo"
)]
async fn echo_post(
    #[request_body(description = "Echo data", required = true)] body: Json<RequestBody>,
) -> Json<String> {
    Json(body.0.data)
}

async fn example_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    next.run(request).await
}

#[tokio::main]
async fn main() {
    // Here you can also add security schemes, other operations, modify internal OpenApi object.
    let oas_builder = OpenApiBuilder::new("Demo", "1.0.0");

    let app = Router::new()
        .route("/echo/get", get(openapi_handler!(echo_get)))
        .route("/echo/post", post(openapi_handler!(echo_post)))
        .route_layer(middleware::from_fn(example_middleware))
        .route_openapi_specification("/openapi", oas_builder)
        .expect("no problem");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}
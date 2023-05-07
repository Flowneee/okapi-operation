use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bytes::{BufMut, BytesMut};
use http::header::ACCEPT;
use http::{header, HeaderMap, HeaderValue, StatusCode};
use okapi::openapi3::OpenApi;
use serde::Serialize;

pub struct Yaml<T>(pub T);

impl<T> IntoResponse for Yaml<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_yaml::to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/x-yaml"),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}

pub async fn axum_yaml_serve_spec(spec: State<OpenApi>, headers: HeaderMap) -> Response {
    match headers.get(ACCEPT).and_then(|h| h.to_str().ok()) {
        Some("yaml") => Yaml(spec.0).into_response(),
        Some("json") => as_json(spec.0),
        Some("*/*") => as_json(spec.0),
        Some("") => as_json(spec.0),
        Some(_) => {
            let status = StatusCode::BAD_REQUEST;
            let headers = [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )];
            let err = format!(
                "Bad Accept header value, should be either 'json' or 'yaml' or '*/*' or empty"
            );
            (status, headers, err).into_response()
        }
        None => as_json(spec.0),
    }
}

fn as_json(spec: OpenApi) -> Response {
    Json(spec).into_response()
}

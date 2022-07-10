# `okapi-operation`

- [`okapi-operation`](#-okapi-operation-)
  * [Example (with axum)](#example-with-axum)
  * [`openapi` macro](#openapi-macro)
    + [Minimal example](#minimal-example)
    + [Operation attributes](#operation-attributes)
    + [External documentation](#external-documentation)
    + [Request parameters](#request-parameters)
      - [Header](#header)
      - [Query](#query)
      - [Path](#path)
      - [Reference](#reference)
    + [Multiple parameters](#multiple-parameters)
    + [Request body](#request-body)
    + [Responses](#responses)
      - [From return type](#from-return-type)
      - [Ignore return type](#ignore-return-type)
      - [Manual definition](#manual-definition)
        * [Single response](#single-response)
        * [From type](#from-type)
      - [Reference](#reference-1)
      - [Multiple responses](#multiple-responses)
    + [Security scheme](#security-scheme)
  * [Building OpenAPI specification](#building-openapi-specification)
  * [Features](#features)
  * [TODO](#todo)

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

## [`openapi`] macro

This macro generate function with name `<original_name>__openapi` of type `fn(&mut Components) -> Result<Operation, anyhow::Error>` ([`OperationGenerator`]), which generate [`okapi::openapi3::Operation`], storing type definitions in provided [`Components`].

If any attribute is missing, it is set to None/false.

Since most attributes taken from OpenAPI specification directly, refer to [OpenAPI website](https://swagger.io/docs/specification/about/) for additional information.

### Minimal example

Macro doesn't have any mandatory attributes.

```rust,compile
# use okapi_operation::*;
#[openapi]
async fn handler() {}
```

### Operation attributes

All attributes is translated into same fields of [`okapi::openapi3::Operation`].

Tags is provided as single string, which later is separated by comma.

```rust,compile
# use okapi_operation::*;
#[openapi(
    summary = "Simple handler",
    description = "Simple handler, demonstrating how to use operation attributes",
    operation_id = "simple",
    tags = "examples,handlers",
    deprecated = false
)]
async fn handler() {}
```

### External documentation

External documentation can be set for operation. It is translated to [`okapi::openapi3::ExternalDocs`].

```rust,compile
# use okapi_operation::*;
#[openapi(
    external_docs(
        url = "https://example.com",
        description = "Example Domain"
    )
)]
async fn handler() {}
```

### Request parameters

Request parameters can be:

* HTTP header (`location: header`);
* query parameter (`?param=value`) (`location: query`);
* part of the path (`/api/user/:id`, where `:id` is parameter) (`location: path`);
* reference to one of the above.

Parameters is defined in `[openapi]` macro. Inferring header from fucntion signature is not supported currently.

This definition translated to [`okapi::openapi3::Parameter`] with [`okapi::openapi3::ParameterValue::Schema`].

#### Header

`header` have following attributes:

* name (string, mandatory);
* description (string, optional);
* required (bool, optional);
* deprecated (bool, optional);
* style (string, optional) - how parameter is serialized (see [OpenAPI docs](https://swagger.io/docs/specification/serialization/));
* schema (path, mandatory) - path to type of parameter.

```rust,compile
# use okapi_operation::*;
#[openapi(
    parameters(
        header(
            name = "x-custom-header",
            description = "Custom header description",
            required = true,
            deprecated = false,
            style = "simple",
            schema = "std::string::String",
        )
    )
)]
async fn handler() {}
```

#### Query

`query` have following attributes:

* name (string, mandatory);
* description (string, optional);
* required (bool, optional);
* deprecated (bool, optional);
* style (string, optional) - how parameter is serialized (https://swagger.io/docs/specification/serialization/);
* explode (bool, optional) - specifies whether arrays and objects should generate separate parameters for each array item or object property;
* allow_empty_value (bool, optional) - allow empty value for this parameter;
* allow_reserved (bool, optional) - allow reserved characters `:/?#[]@!$&'()*+,;=` in parameter;
* schema (path, mandatory) - path to type of parameter.

```rust,compile
# use okapi_operation::*;
#[openapi(
    parameters(
        query(
            name = "page",
            description = "Which page to return",
            required = true,
            deprecated = false,
            style = "simple",
            explode = true,
            allow_empty_value = false,
            allow_reserved = false,
            schema = "std::string::String",
        )
    )
)]
async fn handler() {}
```

#### Path

`path` have following attributes:

* name (string, mandatory);
* description (string, optional);
* deprecated (bool, optional);
* style (string, optional) - how parameter is serialized (https://swagger.io/docs/specification/serialization/);
* schema (path, mandatory) - path to type of parameter.

Unlike header and query parameters, all path parameters is mandatory.

```rust,compile
# use okapi_operation::*;
#[openapi(
    parameters(
        path(
            name = "user_id",
            description = "ID of user",
            deprecated = false,
            style = "simple",
            schema = "std::string::String",
        )
    )
)]
async fn handler() {}
```

#### Reference

```rust,compile
# use okapi_operation::*;
#[openapi(
    parameters(
        reference = "#/components/parameters/ReusableHeader"
    )
)]
async fn handler() {}
```

### Multiple parameters

Specifying multiple parameters is supported:

```rust,compile
# use okapi_operation::*;
#[openapi(
    parameters(
        header(
            name = "x-request-id",
            description = "ID of request for logging",
            required = true,
            deprecated = false,
            style = "simple",
            schema = "std::string::String",
        ),
        header(
            name = "traceparent",
            description = "ID of parent span",
            required = true,
            deprecated = false,
            style = "simple",
            schema = "std::string::String",
        ),
        path(
            name = "user_id",
            description = "ID of user",
            deprecated = false,
            style = "simple",
            schema = "std::string::String",
        ),
        reference = "#/components/parameters/ReusableHeader"
    ),
)]
async fn handler() {}
```

### Request body

Request body is associated with one of function arguments and _by default_ it's schema is inferred from argument type. 

Request body definition have following attributes:

* description (string, optional);
* required (bool, optional);
* content (path, optional) - path to type, which schema should be used. If not speified, argument's type is used.

```rust,compile
# use okapi_operation::*;
# use okapi::schemars::*;
# struct Json<T>(T);
# impl_to_media_types_for_wrapper!(Json<T>, "application/json");
#[derive(JsonSchema)]
struct Request {
    user_id: String
}


#[openapi]
async fn handler(
    #[request_body(
        description = "JSON with user ID",
        required = true,
    )] body: Json<Request>
) {}

#[openapi]
async fn handler_with_request_body_override(
    #[request_body(
        description = "JSON with user ID",
        required = true,
        content = "Json<std::string::String>",
    )] body: Json<Request>
) {}
```

### Responses

Responses can be:

* inferred from return type;
* specified in [`openapi`] macro.

#### From return type

Return type should implement [`ToResponses`] trait.

```rust,compile
# use okapi_operation::*;
# use okapi::schemars::*;
# struct Json<T>(T);
# impl_to_media_types_for_wrapper!(Json<T>, "application/json");
# impl_to_responses_for_wrapper!(Json<T>);
#[derive(JsonSchema)]
struct Response {
    data: String
}

#[openapi]
async fn handler() -> Json<Response> {
# todo!()
}
```

#### Ignore return type

If return type doesn't implement [`ToResponses`], it can be ignored with special attribute `ignore_return_type`:

```rust,compile
# use okapi_operation::*;
#[openapi(
    responses(
        ignore_return_type = true,
    )
)]
async fn handler() -> String {
# todo!()
}
```

#### Manual definition

Manual definition is helpful when you type for some reason doesn't implement [`ToResponses`] or
if you need to specify some responses, which can occur outside handler (in middleware, for example).

##### Single response

Single response define response for a single HTTP status (or pattern). Schema of this response should implement [`ToMediaTypes`].

Single response have following attributes:

* status (string, mandatory) - HTTP status (or pattern like 2XX, 3XX). To define defautl fallback type, use special `default` value;
* description (string, optional);
* content (path, mandatory) - path to type, which provide schemas for this response;
* headers (list, optional) - list of headers (definition is the same as in request parameters). References to header is also allowed.

```rust,compile
# use okapi_operation::*;
# use okapi::schemars::*;
# struct Json<T>(T);
# impl_to_media_types_for_wrapper!(Json<T>, "application/json");
# impl_to_responses_for_wrapper!(Json<T>);
#[derive(JsonSchema)]
struct Response {
    data: String
}

#[openapi(
    responses(
        response(
            status = "200",
            description = "Success",
            content = "Json<Response>",
            headers(
                header(
                    name = "x-custom-message", 
                    description = "Description",
                    required = true,
                    deprecated = false,
                    style = "simple",
                    schema = "std::string::String",
                ),
                reference = "#/components/headers/ReusableHeader"
            ),
        ),
    )
)]
async fn handler() {
# todo!()
}
```


##### From type

Responses can be generated from type, which implement [`ToResponses`]:

```rust,compile
# use okapi_operation::*;
# use okapi::schemars::*;
# struct Json<T>(T);
# impl_to_media_types_for_wrapper!(Json<T>, "application/json");
# impl_to_responses_for_wrapper!(Json<T>);
#[derive(JsonSchema)]
struct Response {
    data: String
}

#[openapi(
    responses(
        from_type = "Json<String>",
    )
)]
async fn handler() {
# todo!()
}
```

`Json<String>` generates single 200 response with JSON with single string.

#### Reference

Reference to response have following attributes:

* status (string, mandatory) - HTTP status (or pattern like 2XX, 3XX). To define defautl fallback type, use special `default` value;
* reference (string, mandatory).

```rust,compile
# use okapi_operation::*;
#[openapi(
    responses(
        reference(
            status = "200",
            reference = "#/components/responses/Reference"
        )
    )
)]
async fn handler() {
# todo!()
}
```

#### Multiple responses

If mutliple manual responses is specified (or specified both return type and manual responses),
they are all merged using [`okapi::merge::merge_responses`]. If multiple responses specified for same HTTP status,
first occurence is used. Responses merged in following order:

* from return type;
* manual single responses;
* references;
* from types.

```rust,compile
# use okapi_operation::*;
# use okapi::schemars::*;
# struct Json<T>(T);
# impl_to_media_types_for_wrapper!(Json<T>, "application/json");
# impl_to_responses_for_wrapper!(Json<T>);
#[derive(JsonSchema)]
struct Response {
    data: String
}

#[openapi(
    responses(
        response(
            status = "500",
            description = "Internal server error",
            content = "Json<String>",
        ),
        reference(
            status = "401",
            reference = "#/components/responses/AuthError"
        ),
        reference(
            status = "403",
            reference = "#/components/responses/AuthError"
        )
    )
)]
async fn handler() -> Json<Response> {
# todo!()
}
```

### Security scheme

Security scheme have following attributes:

* name (string, mandatory) - name of used security scheme;
* scopes (string, optional) - comma separated list of scopes. Have meaning only for `OAuth2` and `OpenID Connect`.

If multiple schemes specified, they are combined as OR. AND is not currently supported.

```rust,compile
# use okapi_operation::*;
#[openapi(
    security(
        security_scheme(
            name = "BasicAuth",
        ),
        security_scheme(
            name = "OAuth2",
            scopes = "scope1,scope2",
        ),
    ),
)]
async fn handler() {}
```

## Building OpenAPI specification

For convenience this crate provide builder-like [`OpenApiBuilder`] type for creating OpenAPI specification:

```rust
# use okapi_operation::*;
# use okapi::schemars::*;
# use http::Method;
# struct Json<T>(T);
# impl_to_media_types_for_wrapper!(Json<T>, "application/json");
# impl_to_responses_for_wrapper!(Json<T>);
#[derive(JsonSchema)]
struct Request {
    user_id: String
}


#[openapi]
async fn handler1(
    #[request_body(
        description = "JSON with user ID",
        required = true,
    )] body: Json<Request>
) {
# todo!()
}

#[openapi]
async fn handler2() -> Json<String> {
# todo!()
}

fn generate_openapi_specification() -> Result<OpenApi, anyhow::Error> {
    OpenApiBuilder::new("Demo", "1.0.0")
        .add_operation("/handle/1", Method::POST, handler1__openapi)?
        .add_operation("/handle/2", Method::GET, handler2__openapi)?
        .generate_spec()
}

assert!(generate_openapi_specification().is_ok());
```

## Features

* `macro`: enables re-import of [`openapi`] macro (enabled by default);
* `axum-integration`: enables integration with [`axum`](https://github.com/tokio-rs/axum) crate (implement traits for certain `axum` types).

## TODO

* [ ] support cookies
* [ ] support examples
* [ ] support inferring schemas of parameters from function definitions
* [ ] ...

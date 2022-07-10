use std::marker::PhantomData;

use axum::{
    handler::Handler,
    http::{Request, Response},
};
use tower::Service;

use crate::OperationGenerator;

/// Wrapper around [`axum::handler::Handler`] with associated OpenAPI [`OperationGenerator`].
pub struct HandlerWithOperation<H, T, B>
where
    H: Handler<T, B>,
{
    pub(super) handler: H,
    pub(super) operation: Option<OperationGenerator>,
    _t: PhantomData<T>,
    _b: PhantomData<B>,
}

impl<H, T, B> From<H> for HandlerWithOperation<H, T, B>
where
    H: Handler<T, B>,
{
    fn from(value: H) -> Self {
        Self {
            handler: value,
            operation: None,
            _t: Default::default(),
            _b: Default::default(),
        }
    }
}

impl<H, T, B> HandlerWithOperation<H, T, B>
where
    H: Handler<T, B>,
{
    pub fn new(handler: H, operation: Option<OperationGenerator>) -> Self {
        Self {
            handler,
            operation,
            _t: PhantomData,
            _b: PhantomData,
        }
    }
}

/// Trait for converting [`axum::handler::Handler`] into wrapper.
pub trait HandlerExt<H, T, B>
where
    H: Handler<T, B>,
{
    fn into_handler_with_operation(self) -> HandlerWithOperation<H, T, B>;

    /// Add OpenAPI operation to handler.
    fn with_openapi(self, operation: OperationGenerator) -> HandlerWithOperation<H, T, B>
    where
        Self: Sized,
    {
        let mut h = self.into_handler_with_operation();
        h.operation = Some(operation);
        h
    }
}

impl<H, T, B> HandlerExt<H, T, B> for H
where
    H: Handler<T, B>,
{
    fn into_handler_with_operation(self) -> HandlerWithOperation<H, T, B> {
        HandlerWithOperation::new(self, None)
    }
}

impl<H, T, B> HandlerExt<H, T, B> for HandlerWithOperation<H, T, B>
where
    H: Handler<T, B>,
{
    fn into_handler_with_operation(self) -> HandlerWithOperation<H, T, B> {
        self
    }
}

/// Wrapper around [`Service`] with associated OpenAPI [`OperationGenerator`].
pub struct ServiceWithOperation<S, ReqBody, RespBody, E>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>, Error = E> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    pub(crate) service: S,
    pub(crate) operation: Option<OperationGenerator>,
    _req_body: PhantomData<ReqBody>,
    _resp_body: PhantomData<RespBody>,
    _e: PhantomData<E>,
}

impl<S, ReqBody, RespBody, E> ServiceWithOperation<S, ReqBody, RespBody, E>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>, Error = E> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    pub(crate) fn new(service: S, operation: Option<OperationGenerator>) -> Self {
        Self {
            service,
            operation,
            _req_body: PhantomData,
            _resp_body: PhantomData,
            _e: PhantomData,
        }
    }
}

impl<S, ReqBody, RespBody, E> From<S> for ServiceWithOperation<S, ReqBody, RespBody, E>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>, Error = E> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    fn from(value: S) -> Self {
        Self::new(value, None)
    }
}

/// Trait for converting [`Service`] into wrapper.
pub trait ServiceExt<S, ReqBody, RespBody, E>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>, Error = E> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    fn into_service_with_operation(self) -> ServiceWithOperation<S, ReqBody, RespBody, E>
where;

    /// Add OpenAPI operation to service.
    fn with_openapi(
        self,
        operation: OperationGenerator,
    ) -> ServiceWithOperation<S, ReqBody, RespBody, E>
    where
        Self: Sized,
    {
        let mut h = self.into_service_with_operation();
        h.operation = Some(operation);
        h
    }
}

impl<S, ReqBody, RespBody, E> ServiceExt<S, ReqBody, RespBody, E> for S
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>, Error = E> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    fn into_service_with_operation(self) -> ServiceWithOperation<S, ReqBody, RespBody, E> {
        ServiceWithOperation::new(self, None)
    }
}

impl<S, ReqBody, RespBody, E> ServiceExt<S, ReqBody, RespBody, E>
    for ServiceWithOperation<S, ReqBody, RespBody, E>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>, Error = E> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    fn into_service_with_operation(self) -> ServiceWithOperation<S, ReqBody, RespBody, E> {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use axum::{body::Body, http::Method, routing::MethodFilter};
    use okapi::openapi3::Operation;
    use tower::service_fn;

    use super::*;
    use crate::{
        axum_integration::{MethodRouter, Router},
        Components,
    };

    fn openapi_generator(_: &mut Components) -> Result<Operation, anyhow::Error> {
        unimplemented!()
    }

    #[test]
    fn handler_with_operation() {
        async fn handler() {}

        let mr: MethodRouter = MethodRouter::new()
            .on(
                MethodFilter::GET,
                (|| async {}).with_openapi(openapi_generator),
            )
            .on(
                MethodFilter::POST,
                handler
                    .with_openapi(openapi_generator)
                    .with_openapi(openapi_generator),
            )
            .on(MethodFilter::PUT, handler)
            .on(MethodFilter::DELETE, || async {});
        let (app, ops) = Router::new().route("/", mr).into_parts();
        assert!(ops.get("/", &Method::GET).is_some());
        assert!(ops.get("/", &Method::POST).is_some());

        let make_service = app.into_make_service();
        let _ = async move {
            axum::Server::bind(&"".parse().unwrap())
                .serve(make_service)
                .await
                .unwrap()
        };
    }

    #[test]
    fn service_with_operation() {
        async fn service(_request: Request<Body>) -> Result<Response<Body>, Infallible> {
            Ok::<_, Infallible>(Response::new(Body::empty()))
        }

        let service2 = service_fn(|_request: Request<Body>| async {
            Ok::<_, Infallible>(Response::new(Body::empty()))
        });

        let mr: MethodRouter = MethodRouter::new()
            .on_service(
                MethodFilter::GET,
                service_fn(service).with_openapi(openapi_generator),
            )
            .on_service(
                MethodFilter::POST,
                service2
                    .clone()
                    .with_openapi(openapi_generator)
                    .with_openapi(openapi_generator),
            )
            .on_service(MethodFilter::PUT, service_fn(service))
            .on_service(MethodFilter::DELETE, service2);
        let (app, ops) = Router::new().route("/", mr).into_parts();
        assert!(ops.get("/", &Method::GET).is_some());
        assert!(ops.get("/", &Method::POST).is_some());

        let make_service = app.into_make_service();
        let _ = async move {
            axum::Server::bind(&"".parse().unwrap())
                .serve(make_service)
                .await
                .unwrap()
        };
    }
}

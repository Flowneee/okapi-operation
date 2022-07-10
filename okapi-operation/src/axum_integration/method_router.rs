use std::{collections::HashMap, convert::Infallible, fmt};

use axum::{
    body::{Body, Bytes, HttpBody},
    error_handling::HandleError,
    handler::Handler,
    http::{Method, Request},
    response::Response,
    routing::{MethodFilter, MethodRouter as AxumMethodRouter, Route},
    BoxError,
};
use tower::{Layer, Service};

use super::handler_traits::{HandlerWithOperation, ServiceWithOperation};
use crate::OperationGenerator;

macro_rules! top_level_service_fn {
    (
        $(#[$m:meta])*
        $name:ident, $method:ident
    ) => {
        $(#[$m])*
        pub fn $name<I, S, ReqBody, ResBody>(svc: I) -> MethodRouter<ReqBody, S::Error>
        where
            I: Into<ServiceWithOperation<S, ReqBody, ResBody, S::Error>>,
            S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
            S::Future: Send + 'static,
            ResBody: HttpBody<Data = Bytes> + Send + 'static,
            ResBody::Error: Into<BoxError>,
        {
            on_service(MethodFilter::$method, svc)
        }
    };
}

macro_rules! top_level_handler_fn {
    (
        $(#[$m:meta])*
        $name:ident, $method:ident
    ) => {
        $(#[$m])*
        pub fn $name<I, H, T, B>(handler: I) -> MethodRouter<B, Infallible>
        where
            I: Into<HandlerWithOperation<H, T, B>>,
            H: Handler<T, B>,
            B: Send + 'static,
            T: 'static,
        {
            on(MethodFilter::$method, handler)
        }
    };
}

macro_rules! chained_service_fn {
    (
        $(#[$m:meta])*
        $name:ident, $method:ident
    ) => {
        $(#[$m])*
        pub fn $name<I, S, ResBody>(self, svc: I) -> Self
        where
            I: Into<ServiceWithOperation<S, ReqBody, ResBody, E>>,
            S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = E>
                + Clone
                + Send
                + 'static,
            S::Future: Send + 'static,
            ResBody: HttpBody<Data = Bytes> + Send + 'static,
            ResBody::Error: Into<BoxError>,
        {
            self.on_service(MethodFilter::$method, svc)
        }
    };
}

macro_rules! chained_handler_fn {
    (
        $(#[$m:meta])*
        $name:ident, $method:ident
    ) => {
        $(#[$m])*
        pub fn $name<I, H, T>(self, handler: I) -> Self
        where
            I: Into<HandlerWithOperation<H, T, B>>,
            H: Handler<T, B>,
            T: 'static,
        {
            self.on(MethodFilter::$method, handler)
        }
    };
}

pub fn on_service<I, S, ReqBody, ResBody>(
    filter: MethodFilter,
    svc: I,
) -> MethodRouter<ReqBody, S::Error>
where
    I: Into<ServiceWithOperation<S, ReqBody, ResBody, S::Error>>,
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    MethodRouter::new().on_service(filter, svc)
}

top_level_service_fn!(delete_service, DELETE);
top_level_service_fn!(get_service, GET);
top_level_service_fn!(head_service, HEAD);
top_level_service_fn!(options_service, OPTIONS);
top_level_service_fn!(patch_service, PATCH);
top_level_service_fn!(post_service, POST);
top_level_service_fn!(put_service, PUT);
top_level_service_fn!(trace_service, TRACE);

pub fn on<I, H, T, B>(filter: MethodFilter, handler: I) -> MethodRouter<B, Infallible>
where
    I: Into<HandlerWithOperation<H, T, B>>,
    H: Handler<T, B>,
    B: Send + 'static,
    T: 'static,
{
    MethodRouter::new().on(filter, handler)
}

top_level_handler_fn!(delete, DELETE);
top_level_handler_fn!(get, GET);
top_level_handler_fn!(head, HEAD);
top_level_handler_fn!(options, OPTIONS);
top_level_handler_fn!(patch, PATCH);
top_level_handler_fn!(post, POST);
top_level_handler_fn!(put, PUT);
top_level_handler_fn!(trace, TRACE);

#[derive(Clone, Default)]
pub(super) struct MethodRouterOperations {
    get: Option<OperationGenerator>,
    head: Option<OperationGenerator>,
    delete: Option<OperationGenerator>,
    options: Option<OperationGenerator>,
    patch: Option<OperationGenerator>,
    post: Option<OperationGenerator>,
    put: Option<OperationGenerator>,
    trace: Option<OperationGenerator>,
}

impl MethodRouterOperations {
    fn on(mut self, filter: MethodFilter, operation: Option<OperationGenerator>) -> Self {
        if filter.contains(MethodFilter::GET) {
            self.get = operation;
        }
        if filter.contains(MethodFilter::HEAD) {
            self.head = operation;
        }
        if filter.contains(MethodFilter::DELETE) {
            self.delete = operation;
        }
        if filter.contains(MethodFilter::OPTIONS) {
            self.options = operation;
        }
        if filter.contains(MethodFilter::PATCH) {
            self.patch = operation;
        }
        if filter.contains(MethodFilter::POST) {
            self.post = operation;
        }
        if filter.contains(MethodFilter::PUT) {
            self.put = operation;
        }
        if filter.contains(MethodFilter::TRACE) {
            self.trace = operation;
        }
        self
    }

    fn merge(self, other: Self) -> Self {
        macro_rules! merge {
            ( $first:ident, $second:ident ) => {
                match ($first, $second) {
                    (Some(_), Some(_)) => panic!(concat!(
                        "Overlapping method operation. Cannot merge two method operation that both define `",
                        stringify!($first),
                        "`"
                    )),
                    (Some(svc), None) => Some(svc),
                    (None, Some(svc)) => Some(svc),
                    (None, None) => None,
                }
            };
        }

        let Self {
            get,
            head,
            delete,
            options,
            patch,
            post,
            put,
            trace,
        } = self;

        let Self {
            get: get_other,
            head: head_other,
            delete: delete_other,
            options: options_other,
            patch: patch_other,
            post: post_other,
            put: put_other,
            trace: trace_other,
        } = other;

        let get = merge!(get, get_other);
        let head = merge!(head, head_other);
        let delete = merge!(delete, delete_other);
        let options = merge!(options, options_other);
        let patch = merge!(patch, patch_other);
        let post = merge!(post, post_other);
        let put = merge!(put, put_other);
        let trace = merge!(trace, trace_other);

        Self {
            get,
            head,
            delete,
            options,
            patch,
            post,
            put,
            trace,
        }
    }

    pub(crate) fn into_map(self) -> HashMap<Method, OperationGenerator> {
        let mut map = HashMap::new();
        if let Some(m) = self.get {
            let _ = map.insert(Method::GET, m);
        }
        if let Some(m) = self.head {
            let _ = map.insert(Method::HEAD, m);
        }
        if let Some(m) = self.delete {
            let _ = map.insert(Method::DELETE, m);
        }
        if let Some(m) = self.options {
            let _ = map.insert(Method::OPTIONS, m);
        }
        if let Some(m) = self.patch {
            let _ = map.insert(Method::PATCH, m);
        }
        if let Some(m) = self.post {
            let _ = map.insert(Method::POST, m);
        }
        if let Some(m) = self.put {
            let _ = map.insert(Method::PUT, m);
        }
        if let Some(m) = self.trace {
            let _ = map.insert(Method::TRACE, m);
        }
        map
    }
}

/// Drop-in replacement for [`axum::routing::MethodRouter`], which supports OpenAPI operations.
pub struct MethodRouter<B = Body, E = Infallible> {
    pub(super) axum_method_router: AxumMethodRouter<B, E>,
    pub(super) operations: MethodRouterOperations,
}

impl<B, E> fmt::Debug for MethodRouter<B, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.axum_method_router.fmt(f)
    }
}

impl<B, E> Default for MethodRouter<B, E>
where
    B: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B, E> MethodRouter<B, E> {
    pub fn new() -> Self {
        Self {
            axum_method_router: AxumMethodRouter::new(),
            operations: Default::default(),
        }
    }

    /// Convert method router into [`axum::routing::MethodRouter`], dropping OpenAPI operations.
    pub fn into_axum(self) -> AxumMethodRouter<B, E> {
        self.axum_method_router
    }
}

impl<B> MethodRouter<B, Infallible>
where
    B: Send + 'static,
{
    pub fn on<I, H, T>(self, filter: MethodFilter, handler: I) -> Self
    where
        I: Into<HandlerWithOperation<H, T, B>>,
        H: Handler<T, B>,
        T: 'static,
    {
        let HandlerWithOperation {
            handler, operation, ..
        } = handler.into();

        Self {
            axum_method_router: self.axum_method_router.on(filter, handler),
            operations: self.operations.on(filter, operation),
        }
    }

    chained_handler_fn!(delete, DELETE);
    chained_handler_fn!(get, GET);
    chained_handler_fn!(head, HEAD);
    chained_handler_fn!(options, OPTIONS);
    chained_handler_fn!(patch, PATCH);
    chained_handler_fn!(post, POST);
    chained_handler_fn!(put, PUT);
    chained_handler_fn!(trace, TRACE);
}

impl<ReqBody, E> MethodRouter<ReqBody, E> {
    pub fn on_service<I, S, ResBody>(self, filter: MethodFilter, svc: I) -> Self
    where
        I: Into<ServiceWithOperation<S, ReqBody, ResBody, E>>,
        S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = E>
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        ResBody: HttpBody<Data = Bytes> + Send + 'static,
        ResBody::Error: Into<BoxError>,
    {
        let ServiceWithOperation {
            service, operation, ..
        } = svc.into();
        Self {
            axum_method_router: self.axum_method_router.on_service(filter, service),
            operations: self.operations.on(filter, operation),
        }
    }

    chained_service_fn!(delete_service, DELETE);
    chained_service_fn!(get_service, GET);
    chained_service_fn!(head_service, HEAD);
    chained_service_fn!(options_service, OPTIONS);
    chained_service_fn!(patch_service, PATCH);
    chained_service_fn!(post_service, POST);
    chained_service_fn!(put_service, PUT);
    chained_service_fn!(trace_service, TRACE);

    /// Установка обработчика для методов, для которых не был найден обработчик.
    ///
    /// См. описание и примеры в [`AxumMethodRouter::fallback`].
    pub fn fallback<S, ResBody>(self, svc: S) -> Self
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = E>
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        ResBody: HttpBody<Data = Bytes> + Send + 'static,
        ResBody::Error: Into<BoxError>,
    {
        Self {
            axum_method_router: self.axum_method_router.fallback(svc),
            ..self
        }
    }

    /// Добавление [`Layer`] к текущему роуту.
    ///
    /// См. описание и примеры в [`AxumMethodRouter::layer`].
    pub fn layer<L, NewReqBody, NewResBody, NewError>(
        self,
        layer: L,
    ) -> MethodRouter<NewReqBody, NewError>
    where
        L: Layer<Route<ReqBody, E>>,
        L::Service: Service<Request<NewReqBody>, Response = Response<NewResBody>, Error = NewError>
            + Clone
            + Send
            + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Future: Send + 'static,
        NewResBody: HttpBody<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.layer(layer),
            operations: self.operations,
        }
    }

    /// Добавление [`Layer`] к текущему роуту.
    ///
    /// В отличии от [`MethodRouter::layer`], `layer` применяется только если удалось сматчить метод.
    ///
    /// См. описание и примеры в [`AxumMethodRouter::route_layer`].
    pub fn route_layer<L, NewResBody>(self, layer: L) -> MethodRouter<ReqBody, E>
    where
        L: Layer<Route<ReqBody, E>>,
        L::Service: Service<Request<ReqBody>, Response = Response<NewResBody>, Error = E>
            + Clone
            + Send
            + 'static,
        <L::Service as Service<Request<ReqBody>>>::Future: Send + 'static,
        NewResBody: HttpBody<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.route_layer(layer),
            operations: self.operations,
        }
    }

    /// Слияние 2 `MethodRouter`.
    ///
    /// См. пример в [`AxumMethodRouter::merge`].
    pub fn merge(self, other: MethodRouter<ReqBody, E>) -> Self {
        MethodRouter {
            axum_method_router: self.axum_method_router.merge(other.axum_method_router),
            operations: self.operations.merge(other.operations),
        }
    }

    /// Применение [`axum::error_handling::HandleErrorLayer`].
    ///
    /// См. пример в [`AxumMethodRouter::handle_error`].
    pub fn handle_error<F, T>(self, f: F) -> MethodRouter<ReqBody, Infallible>
    where
        F: Clone + Send + 'static,
        HandleError<Route<ReqBody, E>, F, T>:
            Service<Request<ReqBody>, Response = Response, Error = Infallible>,
        <HandleError<Route<ReqBody, E>, F, T> as Service<Request<ReqBody>>>::Future: Send,
        T: 'static,
        E: 'static,
        ReqBody: 'static,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.handle_error(f),
            operations: self.operations,
        }
    }
}

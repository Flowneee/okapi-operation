use std::{collections::HashMap, convert::Infallible, fmt};

use axum::{
    body::{Body, HttpBody},
    error_handling::HandleError,
    handler::Handler,
    http::{Method, Request},
    response::IntoResponse,
    routing::{MethodFilter, MethodRouter as AxumMethodRouter, Route},
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
        pub fn $name<I, Svc, S, B, E>(svc: I) -> MethodRouter<S, B, E>
        where
            I: Into<ServiceWithOperation<Svc, B, E>>,
            Svc: Service<Request<B>, Error = E> + Clone + Send + 'static,
            Svc::Response: IntoResponse + 'static,
            Svc::Future: Send + 'static,
            B: HttpBody + Send + 'static,
            S: Clone,
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
        pub fn $name<I, H, T, S, B>(handler: I) -> MethodRouter<S, B, Infallible>
        where
            I: Into<HandlerWithOperation<H, T, S, B>>,
            H: Handler<T, S, B>,
            B: HttpBody + Send + 'static,
            T: 'static,
            S: Clone + Send + Sync + 'static,
        {
            on(MethodFilter::$method, handler)
        }
    };
}

/// Macro for implementing service methods on [`MethodRouter`].
macro_rules! chained_service_fn {
    (
        $(#[$m:meta])*
        $name:ident, $method:ident
    ) => {
        $(#[$m])*
        pub fn $name<I, Svc>(self, svc: I) -> Self
        where
            I: Into<ServiceWithOperation<Svc, B, E>>,
            Svc: Service<Request<B>, Error = E> + Clone + Send + 'static,
            Svc::Response: IntoResponse + 'static,
            Svc::Future: Send + 'static,
        {
            self.on_service(MethodFilter::$method, svc)
        }
    };
}

/// Macro for implementing handler methods on [`MethodRouter`].
macro_rules! chained_handler_fn {
    (
        $(#[$m:meta])*
        $name:ident, $method:ident
    ) => {
        $(#[$m])*
        pub fn $name<I, H, T>(self, handler: I) -> Self
        where
            I: Into<HandlerWithOperation<H, T, S, B>>,
            H: Handler<T, S, B>,
            T: 'static,
            S: Send + Sync + 'static
        {
            self.on(MethodFilter::$method, handler)
        }
    };
}

// TODO: check whether E generic parameter is redundant
pub fn on_service<I, Svc, S, B, E>(filter: MethodFilter, svc: I) -> MethodRouter<S, B, E>
where
    I: Into<ServiceWithOperation<Svc, B, E>>,
    Svc: Service<Request<B>, Error = E> + Clone + Send + 'static,
    Svc::Response: IntoResponse + 'static,
    Svc::Future: Send + 'static,
    B: HttpBody + Send + 'static,
    S: Clone,
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

pub fn on<I, H, T, S, B>(filter: MethodFilter, handler: I) -> MethodRouter<S, B, Infallible>
where
    I: Into<HandlerWithOperation<H, T, S, B>>,
    H: Handler<T, S, B>,
    B: HttpBody + Send + 'static,
    T: 'static,
    S: Clone + Send + Sync + 'static,
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

/// Drop-in replacement for [`axum::routing::MethodRouter`], which supports
/// OpenAPI definitions of handlers or services.
pub struct MethodRouter<S = (), B = Body, E = Infallible> {
    pub(super) axum_method_router: AxumMethodRouter<S, B, E>,
    pub(super) operations: MethodRouterOperations,
}

impl<S, B, E> fmt::Debug for MethodRouter<S, B, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.axum_method_router.fmt(f)
    }
}

impl<S, B, E> Default for MethodRouter<S, B, E>
where
    S: Clone,
    B: HttpBody + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B, E> From<AxumMethodRouter<S, B, E>> for MethodRouter<S, B, E> {
    fn from(value: AxumMethodRouter<S, B, E>) -> Self {
        Self {
            axum_method_router: value,
            operations: Default::default(),
        }
    }
}

impl<S, B> MethodRouter<S, B, Infallible>
where
    S: Clone,
    B: HttpBody + Send + 'static,
{
    pub fn on<I, H, T>(self, filter: MethodFilter, handler: I) -> Self
    where
        I: Into<HandlerWithOperation<H, T, S, B>>,
        H: Handler<T, S, B>,
        T: 'static,
        S: Send + Sync + 'static,
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

    pub fn fallback<H, T>(self, handler: H) -> Self
    where
        H: Handler<T, S, B>,
        T: 'static,
        S: Send + Sync + 'static,
    {
        Self {
            axum_method_router: self.axum_method_router.fallback(handler),
            ..self
        }
    }
}

impl<S, B, E> MethodRouter<S, B, E>
where
    S: Clone,
    B: HttpBody + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            axum_method_router: AxumMethodRouter::new(),
            operations: Default::default(),
        }
    }

    /// Convert method router into [`axum::routing::MethodRouter`], dropping related OpenAPI definitions.
    pub fn into_axum(self) -> AxumMethodRouter<S, B, E> {
        self.axum_method_router
    }

    pub fn on_service<I, Svc>(self, filter: MethodFilter, svc: I) -> Self
    where
        I: Into<ServiceWithOperation<Svc, B, E>>,
        Svc: Service<Request<B>, Error = E> + Clone + Send + 'static,
        Svc::Response: IntoResponse + 'static,
        Svc::Future: Send + 'static,
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

    pub fn fallback_service<Svc>(self, svc: Svc) -> Self
    where
        Svc: Service<Request<B>, Error = E> + Clone + Send + 'static,
        Svc::Response: IntoResponse + 'static,
        Svc::Future: Send + 'static,
    {
        Self {
            axum_method_router: self.axum_method_router.fallback_service(svc),
            ..self
        }
    }

    pub fn layer<L, NewReqBody, NewError>(self, layer: L) -> MethodRouter<S, NewReqBody, NewError>
    where
        L: Layer<Route<B, E>> + Clone + Send + 'static,
        L::Service: Service<Request<NewReqBody>> + Clone + Send + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Error: Into<NewError> + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Future: Send + 'static,
        E: 'static,
        S: 'static,
        NewReqBody: HttpBody + 'static,
        NewError: 'static,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.layer(layer),
            operations: self.operations,
        }
    }

    pub fn route_layer<L, NewResBody>(self, layer: L) -> MethodRouter<S, B, E>
    where
        L: Layer<Route<B, E>> + Clone + Send + 'static,
        L::Service: Service<Request<B>, Error = E> + Clone + Send + 'static,
        <L::Service as Service<Request<B>>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request<B>>>::Future: Send + 'static,
        E: 'static,
        S: 'static,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.route_layer(layer),
            operations: self.operations,
        }
    }

    pub fn merge(self, other: MethodRouter<S, B, E>) -> Self {
        MethodRouter {
            axum_method_router: self.axum_method_router.merge(other.axum_method_router),
            operations: self.operations.merge(other.operations),
        }
    }

    pub fn handle_error<F, T>(self, f: F) -> MethodRouter<S, B, Infallible>
    where
        F: Clone + Send + Sync + 'static,
        HandleError<Route<B, E>, F, T>: Service<Request<B>, Error = Infallible>,
        <HandleError<Route<B, E>, F, T> as Service<Request<B>>>::Future: Send,
        <HandleError<Route<B, E>, F, T> as Service<Request<B>>>::Response: IntoResponse + Send,
        T: 'static,
        E: 'static,
        B: 'static,
        S: 'static,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.handle_error(f),
            operations: self.operations,
        }
    }

    pub fn with_state<S2>(self, state: S) -> MethodRouter<S2, B, E> {
        MethodRouter {
            axum_method_router: self.axum_method_router.with_state(state),
            operations: self.operations,
        }
    }
}

use std::{collections::HashMap, convert::Infallible, fmt};

use axum::{
    error_handling::HandleError,
    extract::Request,
    handler::Handler,
    http::Method,
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
        pub fn $name<I, Svc, S, E>(svc: I) -> MethodRouter<S, E>
        where
            I: Into<ServiceWithOperation<Svc, E>>,
            Svc: Service<Request, Error = E> + Clone + Send + Sync + 'static,
            Svc::Response: IntoResponse + 'static,
            Svc::Future: Send + 'static,
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
        pub fn $name<I, H, T, S>(handler: I) -> MethodRouter<S, Infallible>
        where
            I: Into<HandlerWithOperation<H, T, S>>,
            H: Handler<T, S>,
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
            I: Into<ServiceWithOperation<Svc, E>>,
            Svc: Service<Request, Error = E> + Clone + Send + Sync + 'static,
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
            I: Into<HandlerWithOperation<H, T, S>>,
            H: Handler<T, S>,
            T: 'static,
            S: Send + Sync + 'static
        {
            self.on(MethodFilter::$method, handler)
        }
    };
}

// TODO: check whether E generic parameter is redundant
pub fn on_service<I, Svc, S, E>(filter: MethodFilter, svc: I) -> MethodRouter<S, E>
where
    I: Into<ServiceWithOperation<Svc, E>>,
    Svc: Service<Request, Error = E> + Clone + Send + Sync + 'static,
    Svc::Response: IntoResponse + 'static,
    Svc::Future: Send + 'static,
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

pub fn on<I, H, T, S>(filter: MethodFilter, handler: I) -> MethodRouter<S, Infallible>
where
    I: Into<HandlerWithOperation<H, T, S>>,
    H: Handler<T, S>,
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
        if is_filter_present(filter, MethodFilter::GET) {
            self.get = operation;
        }
        if is_filter_present(filter, MethodFilter::HEAD) {
            self.head = operation;
        }
        if is_filter_present(filter, MethodFilter::DELETE) {
            self.delete = operation;
        }
        if is_filter_present(filter, MethodFilter::OPTIONS) {
            self.options = operation;
        }
        if is_filter_present(filter, MethodFilter::PATCH) {
            self.patch = operation;
        }
        if is_filter_present(filter, MethodFilter::POST) {
            self.post = operation;
        }
        if is_filter_present(filter, MethodFilter::PUT) {
            self.put = operation;
        }
        if is_filter_present(filter, MethodFilter::TRACE) {
            self.trace = operation;
        }
        self
    }

    pub(super) fn merge(self, other: Self) -> Self {
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
pub struct MethodRouter<S = (), E = Infallible> {
    pub(super) axum_method_router: AxumMethodRouter<S, E>,
    pub(super) operations: MethodRouterOperations,
}

impl<S, E> fmt::Debug for MethodRouter<S, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.axum_method_router.fmt(f)
    }
}

impl<S, E> Default for MethodRouter<S, E>
where
    S: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S, E> From<AxumMethodRouter<S, E>> for MethodRouter<S, E> {
    fn from(value: AxumMethodRouter<S, E>) -> Self {
        Self {
            axum_method_router: value,
            operations: Default::default(),
        }
    }
}

impl<S> MethodRouter<S, Infallible>
where
    S: Clone,
{
    pub fn on<I, H, T>(self, filter: MethodFilter, handler: I) -> Self
    where
        I: Into<HandlerWithOperation<H, T, S>>,
        H: Handler<T, S>,
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
        H: Handler<T, S>,
        T: 'static,
        S: Send + Sync + 'static,
    {
        Self {
            axum_method_router: self.axum_method_router.fallback(handler),
            ..self
        }
    }
}

impl<S, E> MethodRouter<S, E>
where
    S: Clone,
{
    pub fn new() -> Self {
        Self {
            axum_method_router: AxumMethodRouter::new(),
            operations: Default::default(),
        }
    }

    /// Convert method router into [`axum::routing::MethodRouter`], dropping related OpenAPI definitions.
    pub fn into_axum(self) -> AxumMethodRouter<S, E> {
        self.axum_method_router
    }

    pub fn on_service<I, Svc>(self, filter: MethodFilter, svc: I) -> Self
    where
        I: Into<ServiceWithOperation<Svc, E>>,
        Svc: Service<Request, Error = E> + Clone + Send + Sync + 'static,
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
        Svc: Service<Request, Error = E> + Clone + Send + Sync + 'static,
        Svc::Response: IntoResponse + 'static,
        Svc::Future: Send + 'static,
    {
        Self {
            axum_method_router: self.axum_method_router.fallback_service(svc),
            ..self
        }
    }

    pub fn layer<L, NewError>(self, layer: L) -> MethodRouter<S, NewError>
    where
        L: Layer<Route<E>> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<NewError> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
        E: 'static,
        S: 'static,
        NewError: 'static,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.layer(layer),
            operations: self.operations,
        }
    }

    pub fn route_layer<L>(self, layer: L) -> MethodRouter<S, E>
    where
        L: Layer<Route<E>> + Clone + Send + Sync + 'static,
        L::Service: Service<Request, Error = E> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
        E: 'static,
        S: 'static,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.route_layer(layer),
            operations: self.operations,
        }
    }

    pub fn merge(self, other: MethodRouter<S, E>) -> Self {
        MethodRouter {
            axum_method_router: self.axum_method_router.merge(other.axum_method_router),
            operations: self.operations.merge(other.operations),
        }
    }

    pub fn handle_error<F, T>(self, f: F) -> MethodRouter<S, Infallible>
    where
        F: Clone + Send + Sync + 'static,
        HandleError<Route<E>, F, T>: Service<Request, Error = Infallible>,
        <HandleError<Route<E>, F, T> as Service<Request>>::Future: Send,
        <HandleError<Route<E>, F, T> as Service<Request>>::Response: IntoResponse + Send,
        T: 'static,
        E: 'static,
        S: 'static,
    {
        MethodRouter {
            axum_method_router: self.axum_method_router.handle_error(f),
            operations: self.operations,
        }
    }

    pub fn with_state<S2>(self, state: S) -> MethodRouter<S2, E> {
        MethodRouter {
            axum_method_router: self.axum_method_router.with_state(state),
            operations: self.operations,
        }
    }
}

fn is_filter_present(lhs: MethodFilter, rhs: MethodFilter) -> bool {
    lhs.or(rhs) == lhs
}

#[test]
fn test_is_filter_present() {
    // Positive tests
    assert!(is_filter_present(
        MethodFilter::DELETE,
        MethodFilter::DELETE
    ));
    assert!(is_filter_present(
        MethodFilter::DELETE.or(MethodFilter::GET),
        MethodFilter::DELETE
    ));
    assert!(is_filter_present(
        MethodFilter::GET.or(MethodFilter::DELETE),
        MethodFilter::DELETE
    ));
    assert!(is_filter_present(
        MethodFilter::DELETE.or(MethodFilter::DELETE),
        MethodFilter::DELETE
    ));

    // Negative tests
    assert!(!is_filter_present(MethodFilter::GET, MethodFilter::DELETE));
}

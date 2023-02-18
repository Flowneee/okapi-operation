use std::{collections::HashMap, convert::Infallible, fmt};

use axum::{
    body::{Body, HttpBody},
    handler::Handler,
    http::{Method, Request},
    response::IntoResponse,
    routing::Route,
    Router as AxumRouter,
};
use tower::{Layer, Service};

use crate::OpenApiBuilder;

use super::{
    get,
    method_router::{MethodRouter, MethodRouterOperations},
    operations::RoutesOperations,
};

/// Drop-in replacement for [`axum::Router`], which supports OpenAPI operations.
///
/// This replacement cannot be used as [`Service`] instead require explicit
/// convertion of this type to `axum::Router`. This is done to ensure that
/// OpenAPI specification generated and mounted.
pub struct Router<S = (), B = Body> {
    axum_router: AxumRouter<S, B>,
    routes_operations_map: HashMap<String, MethodRouterOperations>,
}

impl<S, B> From<AxumRouter<S, B>> for Router<S, B> {
    fn from(value: AxumRouter<S, B>) -> Self {
        Self {
            axum_router: value,
            routes_operations_map: Default::default(),
        }
    }
}

impl<S, B> Default for Router<S, B>
where
    B: HttpBody + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> fmt::Debug for Router<S, B>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.axum_router.fmt(f)
    }
}

impl<S, B> Router<S, B>
where
    B: HttpBody + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// Create new router.
    pub fn new() -> Self {
        Self {
            axum_router: AxumRouter::new(),
            routes_operations_map: HashMap::new(),
        }
    }

    /// Add another route to the router.
    ///
    /// This method works for both [`MethodRouter`] and one from axum.
    ///
    /// For details see [`axum::Router::route`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use okapi_operation::{*, axum_integration::*};
    /// #[openapi]
    /// async fn handler() {}
    ///
    /// let app = Router::new().route("/", get(openapi_handler!(handler)));
    /// # async {
    /// # let (app, _) = app.into_parts();
    /// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
    /// # };
    /// ```
    pub fn route<R>(mut self, path: &str, method_router: R) -> Self
    where
        R: Into<MethodRouter<S, B>>,
    {
        let method_router = method_router.into();
        self.routes_operations_map
            .insert(path.into(), method_router.operations);
        Self {
            axum_router: self
                .axum_router
                .route(path, method_router.axum_method_router),
            ..self
        }
    }

    /// Add another route to the router that calls a [`Service`].
    ///
    /// For details see [`axum::Router::route_service`].
    ///
    /// # Example
    ///
    /// TODO
    pub fn route_service<Svc>(self, path: &str, service: Svc) -> Self
    where
        Svc: Service<Request<B>, Error = Infallible> + Clone + Send + 'static,
        Svc::Response: IntoResponse,
        Svc::Future: Send + 'static,
    {
        Self {
            axum_router: self.axum_router.route_service(path, service),
            ..self
        }
    }

    /// Nest a router at some path.
    ///
    /// This method works for both [`Router`] and one from axum.
    ///
    /// For details see [`axum::Router::nest`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use okapi_operation::{*, axum_integration::*};
    /// #[openapi]
    /// async fn handler() {}
    /// let handler_router = Router::new().route("/", get(openapi_handler!(handler)));
    /// let app = Router::new().nest("/handle", handler_router);
    /// # async {
    /// # let (app, _) = app.into_parts();
    /// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
    /// # };
    /// ```
    pub fn nest<R>(mut self, path: &str, router: R) -> Self
    where
        R: Into<Router<S, B>>,
    {
        let router = router.into();
        for (inner_path, operation) in router.routes_operations_map.into_iter() {
            let _ = self
                .routes_operations_map
                .insert(format!("{}{}", path, inner_path), operation);
        }
        Self {
            axum_router: self.axum_router.nest(path, router.axum_router),
            ..self
        }
    }

    /// Like `nest`, but accepts an arbitrary [`Service`].
    ///
    /// For details see [`axum::Router::nest_service`].
    pub fn nest_service<Svc>(self, path: &str, svc: Svc) -> Self
    where
        Svc: Service<Request<B>, Error = Infallible> + Clone + Send + 'static,
        Svc::Response: IntoResponse,
        Svc::Future: Send + 'static,
    {
        Self {
            axum_router: self.axum_router.nest_service(path, svc),
            ..self
        }
    }

    /// Merge two routers into one.
    ///
    /// This method works for both [`Router`] and one from axum.
    ///
    /// For details see [`axum::Router::merge`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use okapi_operation::{*, axum_integration::*};
    /// #[openapi]
    /// async fn handler() {}
    /// let handler_router = Router::new().route("/another_handler", get(openapi_handler!(handler)));
    /// let app = Router::new().route("/", get(openapi_handler!(handler))).merge(handler_router);
    /// # async {
    /// # let (app, _) = app.into_parts();
    /// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
    /// # };
    /// ```
    pub fn merge<R>(mut self, other: R) -> Self
    where
        R: Into<Router<S, B>>,
    {
        let other = other.into();
        self.routes_operations_map
            .extend(other.routes_operations_map);
        Self {
            axum_router: self.axum_router.merge(other.axum_router),
            ..self
        }
    }

    /// Apply a [`tower::Layer`] to the router.
    ///
    /// For details see [`axum::Router::layer`].
    pub fn layer<L, NewReqBody, NewResBody>(self, layer: L) -> Router<S, NewReqBody>
    where
        L: Layer<Route<B>> + Clone + Send + 'static,
        L::Service: Service<Request<NewReqBody>> + Clone + Send + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Future: Send + 'static,
        NewReqBody: HttpBody + 'static,
    {
        Router {
            axum_router: self.axum_router.layer(layer),
            routes_operations_map: self.routes_operations_map,
        }
    }

    /// Apply a [`tower::Layer`] to the router that will only run if the request matches a route.
    ///
    /// For details see [`axum::Router::route_layer`].
    pub fn route_layer<L, NewResBody>(self, layer: L) -> Self
    where
        L: Layer<Route<B>> + Clone + Send + 'static,
        L::Service: Service<Request<B>> + Clone + Send + 'static,
        <L::Service as Service<Request<B>>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request<B>>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request<B>>>::Future: Send + 'static,
    {
        Router {
            axum_router: self.axum_router.route_layer(layer),
            routes_operations_map: self.routes_operations_map,
        }
    }

    // TODO: somehow mount openapi doc from this handler
    /// Add a fallback [`Service`] to the router.
    ///
    /// For details see [`axum::Router::fallback_service`].
    ///
    /// # Note
    ///
    /// This method doesn't add anything to OpenaAPI spec.
    pub fn fallback<H, T>(self, handler: H) -> Self
    where
        H: Handler<T, S, B>,
        T: 'static,
    {
        Router {
            axum_router: self.axum_router.fallback(handler),
            ..self
        }
    }

    /// Add a fallback [`Service`] to the router.
    ///
    /// For details see [`axum::Router::fallback_service`].
    ///
    /// # Note
    ///
    /// This method doesn't add anything to OpenaAPI spec.
    pub fn fallback_service<Svc>(self, svc: Svc) -> Self
    where
        Svc: Service<Request<B>, Error = Infallible> + Clone + Send + 'static,
        Svc::Response: IntoResponse,
        Svc::Future: Send + 'static,
    {
        Router {
            axum_router: self.axum_router.fallback_service(svc),
            ..self
        }
    }

    /// Provide the state for the router.
    ///
    /// For details see [`axum::Router::with_state`].
    pub fn with_state<S2>(self, state: S) -> Router<S2, B> {
        Router {
            axum_router: self.axum_router.with_state(state),
            routes_operations_map: self.routes_operations_map,
        }
    }

    /// Separate router into [`axum::Router`] and list of operations.
    pub fn into_parts(self) -> (AxumRouter<S, B>, RoutesOperations) {
        (
            self.axum_router,
            RoutesOperations::new(self.routes_operations_map),
        )
    }

    /// Get inner [`axum::Router`].
    pub fn axum_router(&self) -> AxumRouter<S, B> {
        self.axum_router.clone()
    }

    /// Get list of operations.
    pub fn routes_operations(&self) -> RoutesOperations {
        RoutesOperations::new(self.routes_operations_map.clone())
    }

    // TODO: refactor, I don't like this API
    /// Generate OpenAPI specification, mount it to inner router and return [`axum::Router`].
    ///
    /// This method is just for convenience and should be used after all routes mounted to root router.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use okapi_operation::{*, axum_integration::*};
    /// #[openapi]
    /// async fn handler() {}
    ///
    /// let app = Router::new().route("/", get(openapi_handler!(handler)));
    /// # async {
    /// let oas_builder = OpenApiBuilder::new("Demo", "1.0.0");
    /// let app = app.route_openapi_specification("/openapi", oas_builder).expect("ok");
    /// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
    /// # };
    /// ```
    pub fn route_openapi_specification(
        mut self,
        path: &str,
        mut openapi_builder: OpenApiBuilder,
    ) -> Result<AxumRouter<S, B>, anyhow::Error> {
        let mut routes = self.routes_operations().openapi_operation_generators();
        let _ = routes.insert(
            (path.to_string(), Method::GET),
            super::serve_openapi_spec__openapi,
        );
        let spec = openapi_builder
            .add_operations(routes.into_iter().map(|((x, y), z)| (x, y, z)))?
            .generate_spec()?;
        self = self.route(path, get(super::serve_openapi_spec).with_state(spec));
        Ok(self.axum_router)
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::Method, routing::get as axum_get};
    use okapi::openapi3::Operation;

    use super::*;
    use crate::{
        axum_integration::{get, HandlerExt},
        Components,
    };

    fn openapi_generator(_: &mut Components) -> Result<Operation, anyhow::Error> {
        unimplemented!()
    }

    #[test]
    fn mount_axum_types() {
        let axum_router = AxumRouter::new().route("/get", axum_get(|| async {}));
        let (app, meta) = Router::new()
            .route("/", axum_get(|| async {}))
            .nest("/nested", axum_router.clone())
            .merge(axum_router)
            .into_parts();
        assert!(meta.0.is_empty());
        let make_service = app.into_make_service();
        let _ = async move {
            axum::Server::bind(&"".parse().unwrap())
                .serve(make_service)
                .await
                .unwrap()
        };
    }

    #[test]
    fn mount() {
        let router = Router::new().route("/get", get(|| async {})).route(
            "/get_with_spec",
            get((|| async {}).with_openapi(openapi_generator)),
        );
        let router2 = Router::new().route("/get", get(|| async {})).route(
            "/get_with_spec",
            get((|| async {}).with_openapi(openapi_generator)),
        );
        let (app, ops) = Router::new()
            .route("/", get(|| async {}))
            .nest("/nested", router)
            .merge(router2)
            .into_parts();

        assert!(ops.get_path("/").is_none());
        assert!(ops.get_path("/get").is_none());
        assert!(ops.get_path("/nested/get").is_none());

        assert!(ops.get_path("/get_with_spec").is_some());
        assert!(ops.get("/get_with_spec", &Method::GET).is_some());
        assert!(ops.get("/get_with_spec", &Method::POST).is_none());
        assert!(ops.get_path("/nested/get_with_spec").is_some());
        assert!(ops.get("/nested/get_with_spec", &Method::GET).is_some());
        assert!(ops.get("/nested/get_with_spec", &Method::POST).is_none());

        let make_service = app.into_make_service();
        let _ = async move {
            axum::Server::bind(&"".parse().unwrap())
                .serve(make_service)
                .await
                .unwrap()
        };
    }
}

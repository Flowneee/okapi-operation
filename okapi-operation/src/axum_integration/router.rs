use std::{collections::HashMap, convert::Infallible, fmt};

use axum::{
    Router as AxumRouter, extract::Request, handler::Handler, http::Method, response::IntoResponse,
    routing::Route,
};
use tower::{Layer, Service};

use super::{
    get,
    method_router::{MethodRouter, MethodRouterOperations},
    operations::RoutesOperations,
    utils::convert_axum_path_to_openapi,
};
use crate::OpenApiBuilder;

pub const DEFAULT_OPENAPI_PATH: &str = "/openapi";

/// Drop-in replacement for [`axum::Router`], which supports OpenAPI operations.
///
/// This replacement cannot be used as [`Service`] instead require explicit
/// convertion of this type to `axum::Router`. This is done to ensure that
/// OpenAPI specification generated and mounted.
pub struct Router<S = ()> {
    axum_router: AxumRouter<S>,
    routes_operations_map: HashMap<String, MethodRouterOperations>,
    openapi_builder_template: OpenApiBuilder,
}

impl<S> From<AxumRouter<S>> for Router<S> {
    fn from(value: AxumRouter<S>) -> Self {
        Self {
            axum_router: value,
            routes_operations_map: Default::default(),
            openapi_builder_template: OpenApiBuilder::default(),
        }
    }
}

impl<S> Clone for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            axum_router: self.axum_router.clone(),
            routes_operations_map: self.routes_operations_map.clone(),
            openapi_builder_template: self.openapi_builder_template.clone(),
        }
    }
}

impl<S> Default for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> fmt::Debug for Router<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.axum_router.fmt(f)
    }
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Create new router.
    pub fn new() -> Self {
        Self {
            axum_router: AxumRouter::new(),
            routes_operations_map: HashMap::new(),
            openapi_builder_template: OpenApiBuilder::default(),
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
    /// # let listener = tokio::net::TcpListener::bind("").await.unwrap();
    /// # axum::serve(listener, app.into_make_service()).await.unwrap()
    /// # };
    /// ```
    pub fn route<R>(mut self, path: &str, method_router: R) -> Self
    where
        R: Into<MethodRouter<S>>,
    {
        let method_router = method_router.into();

        // Merge operations
        let s = self.routes_operations_map.entry(path.into()).or_default();
        *s = s.clone().merge(method_router.operations);

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
        Svc: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
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
    /// # let listener = tokio::net::TcpListener::bind("").await.unwrap();
    /// # axum::serve(listener, app.into_parts().0.into_make_service()).await.unwrap()
    /// # };
    /// ```
    pub fn nest<R>(mut self, path: &str, router: R) -> Self
    where
        R: Into<Router<S>>,
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
        Svc: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
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
    /// # let listener = tokio::net::TcpListener::bind("").await.unwrap();
    /// # axum::serve(listener, app.into_make_service()).await.unwrap()
    /// # };
    /// ```
    pub fn merge<R>(mut self, other: R) -> Self
    where
        R: Into<Router<S>>,
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
    pub fn layer<L>(self, layer: L) -> Router<S>
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        Router {
            axum_router: self.axum_router.layer(layer),
            routes_operations_map: self.routes_operations_map,
            openapi_builder_template: self.openapi_builder_template,
        }
    }

    /// Apply a [`tower::Layer`] to the router that will only run if the request matches a route.
    ///
    /// For details see [`axum::Router::route_layer`].
    pub fn route_layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        Router {
            axum_router: self.axum_router.route_layer(layer),
            routes_operations_map: self.routes_operations_map,
            openapi_builder_template: self.openapi_builder_template,
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
        H: Handler<T, S>,
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
        Svc: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
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
    pub fn with_state<S2>(self, state: S) -> Router<S2> {
        Router {
            axum_router: self.axum_router.with_state(state),
            routes_operations_map: self.routes_operations_map,
            openapi_builder_template: self.openapi_builder_template,
        }
    }

    /// Separate router into [`axum::Router`] and list of operations.
    pub fn into_parts(self) -> (AxumRouter<S>, RoutesOperations) {
        (
            self.axum_router,
            RoutesOperations::new(self.routes_operations_map),
        )
    }

    /// Get inner [`axum::Router`].
    pub fn axum_router(&self) -> AxumRouter<S> {
        self.axum_router.clone()
    }

    /// Get list of operations.
    pub fn routes_operations(&self) -> RoutesOperations {
        RoutesOperations::new(self.routes_operations_map.clone())
    }

    /// Generate [`OpenApiBuilder`] from current router.
    ///
    /// Generated builder will be based on current builder template,
    /// have all routes and types, present in this router.
    ///
    /// If template was not set, then [`OpenApiBuilder::default()`] is used.
    pub fn generate_openapi_builder(&self) -> OpenApiBuilder {
        let routes = self.routes_operations().openapi_operation_generators();
        let mut builder = self.openapi_builder_template.clone();
        // Don't use try_operations since duplicates should be checked
        // when mounting route to axum router.
        builder.operations(
            routes
                .into_iter()
                .map(|((x, y), z)| (convert_axum_path_to_openapi(&x), y, z)),
        );
        builder
    }

    /// Set [`OpenApiBuilder`] template for this router.
    ///
    /// By default [`OpenApiBuilder::default()`] is used.
    pub fn set_openapi_builder_template(&mut self, builder: OpenApiBuilder) -> &mut Self {
        self.openapi_builder_template = builder;
        self
    }

    /// Update [`OpenApiBuilder`] template of this router.
    ///
    /// By default [`OpenApiBuilder::default()`] is used.
    pub fn update_openapi_builder_template<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut OpenApiBuilder),
    {
        f(&mut self.openapi_builder_template);
        self
    }

    /// Get mutable reference to [`OpenApiBuilder`] template of this router.
    ///
    /// By default [`OpenApiBuilder::default()`] is set.
    pub fn openapi_builder_template_mut(&mut self) -> &mut OpenApiBuilder {
        &mut self.openapi_builder_template
    }

    /// Generate OpenAPI specification, mount it to inner router and return inner [`axum::Router`].
    ///
    /// Specification is based on [`OpenApiBuilder`] template, if one was set previously.
    /// If template was not set, then [`OpenApiBuilder::default()`] is used.
    ///
    /// Note that passed `title` and `version` will override same values in OpenAPI builder template.
    ///
    /// By default specification served at [`DEFAULT_OPENAPI_PATH`] (`/openapi`).
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
    /// let app = app.finish_openapi("/openapi", "Demo", "1.0.0").expect("ok");
    /// # let listener = tokio::net::TcpListener::bind("").await.unwrap();
    /// # axum::serve(listener, app.into_make_service()).await.unwrap()
    /// # };
    /// ```
    pub fn finish_openapi<'a>(
        mut self,
        serve_path: impl Into<Option<&'a str>>,
        title: impl Into<String>,
        version: impl Into<String>,
    ) -> Result<AxumRouter<S>, anyhow::Error> {
        let serve_path = serve_path.into().unwrap_or(DEFAULT_OPENAPI_PATH);

        // Don't use try_operation since duplicates should be checked
        // when mounting route to axum router.
        let spec = self
            .generate_openapi_builder()
            .operation(
                convert_axum_path_to_openapi(serve_path),
                Method::GET,
                super::serve_openapi_spec__openapi,
            )
            .title(title)
            .version(version)
            .build()?;

        self = self.route(serve_path, get(super::serve_openapi_spec).with_state(spec));

        Ok(self.axum_router)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::let_underscore_future)]

    use axum::{http::Method, routing::get as axum_get};
    use okapi::openapi3::Operation;
    use tokio::net::TcpListener;

    use super::*;
    use crate::{
        Components,
        axum_integration::{HandlerExt, get, post},
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
            let listener = TcpListener::bind("").await.unwrap();
            axum::serve(listener, make_service).await.unwrap()
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
            .route(
                "/my_path",
                get((|| async {}).with_openapi(openapi_generator)),
            )
            .route(
                "/my_path",
                post((|| async {}).with_openapi(openapi_generator)),
            )
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
        assert!(ops.get("/nested/get_with_spec", &Method::POST).is_none());

        assert!(ops.get("/my_path", &Method::GET).is_some());
        assert!(ops.get("/my_path", &Method::POST).is_some());

        let make_service = app.into_make_service();
        let _ = async move {
            let listener = TcpListener::bind("").await.unwrap();
            axum::serve(listener, make_service).await.unwrap()
        };
    }
}

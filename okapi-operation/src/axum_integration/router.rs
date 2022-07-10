use std::{collections::HashMap, convert::Infallible, fmt};

use axum::{
    body::{Body, Bytes, HttpBody},
    http::{Method, Request},
    response::Response,
    routing::Route,
    Extension, Router as AxumRouter,
};
use tower::{BoxError, Layer, Service};

use crate::OpenApiBuilder;

use super::{
    get,
    method_router::{MethodRouter, MethodRouterOperations},
    operations::RoutesOperations,
};

/// Drop-in replacement for [`axum::Router`], which supports OpenAPI operations.
pub struct Router<B = Body> {
    axum_router: AxumRouter<B>,
    routes_operations_map: HashMap<String, MethodRouterOperations>,
}

impl<B> From<AxumRouter<B>> for Router<B> {
    fn from(value: AxumRouter<B>) -> Self {
        Self {
            axum_router: value,
            routes_operations_map: Default::default(),
        }
    }
}

impl<B> Default for Router<B>
where
    B: HttpBody + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B> fmt::Debug for Router<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.axum_router.fmt(f)
    }
}

impl<B> Router<B>
where
    B: HttpBody + Send + 'static,
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
    pub fn route(mut self, path: &str, method_router: MethodRouter<B, Infallible>) -> Self {
        self.routes_operations_map
            .insert(path.into(), method_router.operations);
        Self {
            axum_router: self
                .axum_router
                .route(path, method_router.axum_method_router),
            ..self
        }
    }

    /// Add another axum route to the router.
    ///
    /// For details see [`axum::Router::route`].
    pub fn route_axum<T>(self, path: &str, service: T) -> Self
    where
        T: Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        T::Future: Send + 'static,
    {
        Self {
            axum_router: self.axum_router.route(path, service),
            ..self
        }
    }

    /// Nest a router at some path.
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
    pub fn nest(mut self, path: &str, router: Self) -> Self {
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

    /// Nest an [`axum::Router`] at some path.
    ///
    /// For details see [`axum::Router::nest`].
    pub fn nest_axum<T>(self, path: &str, svc: T) -> Self
    where
        T: Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        T::Future: Send + 'static,
    {
        Self {
            axum_router: self.axum_router.nest(path, svc),
            ..self
        }
    }

    /// Merge two routers into one.
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
    pub fn merge(mut self, other: Router<B>) -> Self {
        self.routes_operations_map
            .extend(other.routes_operations_map);
        Self {
            axum_router: self.axum_router.merge(other.axum_router),
            ..self
        }
    }

    /// Merge current router and [`axum::Router`] into one.
    ///
    /// For details see [`axum::Router::merge`].
    pub fn merge_axum(self, other: AxumRouter<B>) -> Self {
        Self {
            axum_router: self.axum_router.merge(other),
            ..self
        }
    }

    /// Apply a [`tower::Layer`] to the router.
    ///
    /// For details see [`axum::Router::layer`].
    pub fn layer<L, NewReqBody, NewResBody>(self, layer: L) -> Router<NewReqBody>
    where
        L: Layer<Route<B>>,
        L::Service: Service<Request<NewReqBody>, Response = Response<NewResBody>, Error = Infallible>
            + Clone
            + Send
            + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Future: Send + 'static,
        NewResBody: HttpBody<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
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
        L: Layer<Route<B>>,
        L::Service: Service<Request<B>, Response = Response<NewResBody>, Error = Infallible>
            + Clone
            + Send
            + 'static,
        <L::Service as Service<Request<B>>>::Future: Send + 'static,
        NewResBody: HttpBody<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
    {
        Router {
            axum_router: self.axum_router.route_layer(layer),
            routes_operations_map: self.routes_operations_map,
        }
    }

    /// Add a fallback service to the router.
    ///
    /// For details see [`axum::Router::fallback`].
    pub fn fallback<T>(self, svc: T) -> Self
    where
        T: Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        T::Future: Send + 'static,
    {
        Router {
            axum_router: self.axum_router.fallback(svc),
            ..self
        }
    }

    /// Separate router into [`axum::Router`] and list of operations.
    pub fn into_parts(self) -> (AxumRouter<B>, RoutesOperations) {
        (
            self.axum_router,
            RoutesOperations::new(self.routes_operations_map),
        )
    }

    /// Get inner [`axum::Router`].
    pub fn axum_router(&self) -> AxumRouter<B> {
        self.axum_router.clone()
    }

    /// Get list of operations.
    pub fn routes_operations(&self) -> RoutesOperations {
        RoutesOperations::new(self.routes_operations_map.clone())
    }

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
    ) -> Result<AxumRouter<B>, anyhow::Error> {
        let mut routes = self.routes_operations().openapi_operation_generators();
        let _ = routes.insert(
            (path.to_string(), Method::GET),
            super::serve_openapi_spec__openapi,
        );
        let spec = openapi_builder
            .add_operations(routes.into_iter().map(|((x, y), z)| (x, y, z)))?
            .generate_spec()?;
        self = self.route(
            path,
            get(super::serve_openapi_spec).route_layer(Extension(spec)),
        );
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
    fn mount_axum() {
        let axum_router = AxumRouter::new().route("/get", axum_get(|| async {}));
        let (app, meta) = Router::new()
            .route_axum("/", axum_get(|| async {}))
            .nest_axum("/nested", axum_router.clone())
            .merge_axum(axum_router)
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

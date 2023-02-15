use http::Method;
use okapi::openapi3::{Info, OpenApi, SecurityRequirement, SecurityScheme};

use crate::{components::Components, utils::convert_axum_path_to_openapi, OperationGenerator};

/// OpenAPI specificatrion builder.
pub struct OpenApiBuilder {
    spec: OpenApi,
    components: Components,
}

impl OpenApiBuilder {
    /// Create new builder with specified title and version.
    pub fn new(title: &str, spec_version: &str) -> Self {
        let spec = OpenApi {
            openapi: OpenApi::default_version(),
            info: Info {
                title: title.into(),
                version: spec_version.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        Self {
            spec,
            components: Components::new(Default::default()),
        }
    }

    /// Alter default [`Components`].
    ///
    /// ## NOTE
    ///
    /// This will override existing components in builder. use this before adding anything to
    /// the builder.
    pub fn set_components(&mut self, new_components: Components) -> &mut Self {
        self.components = new_components;
        self
    }

    /// Set OpenAPI version (should be 3.0.x).
    pub fn set_openapi_version(&mut self, version: String) -> &mut Self {
        self.spec.openapi = version;
        self
    }

    /// Access to inner [`okapi::openapi3::OpenApi`].
    pub fn spec_mut(&mut self) -> &mut OpenApi {
        &mut self.spec
    }

    /// Add security scheme definition.
    pub fn add_security_def<N>(&mut self, name: N, sec: SecurityScheme) -> &mut Self
    where
        N: Into<String>,
    {
        self.components.add_security(name, sec);
        self
    }

    /// Apply security scheme globally.
    pub fn apply_global_security<N, S>(&mut self, name: N, scopes: S) -> &mut Self
    where
        N: Into<String>,
        S: IntoIterator<Item = String>,
    {
        let mut sec = SecurityRequirement::new();
        sec.insert(name.into(), scopes.into_iter().collect());
        self.spec.security.push(sec);
        self
    }

    /// Add single operation.
    pub fn add_operation(
        &mut self,
        path: &str,
        method: Method,
        generator: OperationGenerator,
    ) -> Result<&mut Self, anyhow::Error> {
        let operation_schema = generator(&mut self.components)?;
        let path = self.spec.paths.entry(path.into()).or_default();
        if method == Method::DELETE {
            path.delete = Some(operation_schema);
        } else if method == Method::GET {
            path.get = Some(operation_schema);
        } else if method == Method::HEAD {
            path.head = Some(operation_schema);
        } else if method == Method::OPTIONS {
            path.options = Some(operation_schema);
        } else if method == Method::PATCH {
            path.patch = Some(operation_schema);
        } else if method == Method::POST {
            path.post = Some(operation_schema);
        } else if method == Method::PUT {
            path.put = Some(operation_schema);
        } else if method == Method::TRACE {
            path.trace = Some(operation_schema);
        } else {
            return Err(anyhow::anyhow!("Unsupported method {}", method));
        }
        Ok(self)
    }

    /// Add multiple operations.
    pub fn add_operations(
        &mut self,
        operations: impl Iterator<Item = (String, Method, OperationGenerator)>,
    ) -> Result<&mut Self, anyhow::Error> {
        for (path, method, f) in operations {
            self.add_operation(&convert_axum_path_to_openapi(&path), method, f)?;
        }
        Ok(self)
    }

    /// Generate [`okapi::openapi3::OpenApi`] specification.
    pub fn generate_spec(&mut self) -> Result<OpenApi, anyhow::Error> {
        let mut spec = self.spec.clone();
        spec.components = Some(self.components.okapi_components()?);
        Ok(spec)
    }
}

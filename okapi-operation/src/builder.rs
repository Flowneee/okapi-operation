use std::collections::HashSet;

use anyhow::{Context, anyhow, bail};
use http::Method;
use indexmap::IndexMap;
use okapi::openapi3::{
    Contact, ExternalDocs, License, OpenApi, SecurityRequirement, SecurityScheme, Server, Tag,
};

use crate::{OperationGenerator, components::Components};

#[derive(Clone)]
pub struct BuilderOptions {
    pub infer_operation_id: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for BuilderOptions {
    fn default() -> Self {
        Self {
            infer_operation_id: false,
        }
    }
}

impl BuilderOptions {
    pub fn infer_operation_id(&self) -> bool {
        self.infer_operation_id
    }
}

/// OpenAPI specificatrion builder.
#[derive(Clone)]
pub struct OpenApiBuilder {
    spec: OpenApi,
    components: Components,
    operations: IndexMap<(String, Method), OperationGenerator>,
    known_operation_ids: HashSet<String>, // Used to validate operation ids
    builder_options: BuilderOptions,
}

impl Default for OpenApiBuilder {
    fn default() -> Self {
        let spec = OpenApi {
            openapi: OpenApi::default_version(),
            ..Default::default()
        };
        Self {
            spec,
            components: Components::new(Default::default()),
            operations: IndexMap::new(),
            known_operation_ids: Default::default(),
            builder_options: Default::default(),
        }
    }
}

impl OpenApiBuilder {
    /// Create new builder with specified title and version
    pub fn new(title: &str, version: &str) -> Self {
        let mut this = Self::default();
        this.title(title);
        this.version(version);
        this
    }

    /// Alter default [`Components`].
    ///
    /// ## NOTE
    ///
    /// This will override existing components in builder. Use this before adding anything to
    /// the builder.
    pub fn set_components(&mut self, new_components: Components) -> &mut Self {
        self.components = new_components;
        self
    }

    /// Add single operation.
    ///
    /// Throws an error if (path, method) pair is already present.
    pub fn try_operation<T>(
        &mut self,
        path: T,
        method: Method,
        generator: OperationGenerator,
    ) -> Result<&mut Self, anyhow::Error>
    where
        T: Into<String>,
    {
        let path = path.into();
        if self
            .operations
            .insert((path.clone(), method.clone()), generator)
            .is_some()
        {
            bail!("{method} {path} is already present in specification");
        };
        Ok(self)
    }

    /// Add multiple operations.
    ///
    /// Throws an error if any (path, method) pair is already present.
    pub fn try_operations<I, S>(&mut self, operations: I) -> Result<&mut Self, anyhow::Error>
    where
        I: Iterator<Item = (S, Method, OperationGenerator)>,
        S: Into<String>,
    {
        for (path, method, f) in operations {
            self.try_operation(path, method, f)?;
        }
        Ok(self)
    }

    /// Add single operation.
    ///
    /// Replaces operation if (path, method) pair is already present.
    pub fn operation<T>(
        &mut self,
        path: T,
        method: Method,
        generator: OperationGenerator,
    ) -> &mut Self
    where
        T: Into<String>,
    {
        let _ = self.try_operation(path, method, generator);
        self
    }

    /// Add multiple operations.
    ///
    /// Replaces operation if (path, method) pair is already present.
    pub fn operations<I, S>(&mut self, operations: I) -> &mut Self
    where
        I: Iterator<Item = (S, Method, OperationGenerator)>,
        S: Into<String>,
    {
        for (path, method, f) in operations {
            self.operation(path, method, f);
        }
        self
    }

    /// Access inner [`okapi::openapi3::OpenApi`].
    ///
    /// **Warning!** This allows raw access to underlying `OpenApi` object,
    /// which might break generated specification.
    ///
    /// # NOTE
    ///
    /// Components are overwritten on building specification.
    pub fn spec_mut(&mut self) -> &mut OpenApi {
        &mut self.spec
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

    /// Infer the operation id for every operation based on the function name.
    ///
    /// If the operation_id is specified in the macro, it will replace the inferred name.
    pub fn set_infer_operation_id(&mut self, value: bool) -> &mut Self {
        self.builder_options.infer_operation_id = value;
        self
    }

    /// Add single operation.
    pub fn add_operation(
        &mut self,
        path: &str,
        method: Method,
        generator: OperationGenerator,
    ) -> Result<&mut Self, anyhow::Error> {
        let operation_schema = generator(&mut self.components, &self.builder_options)?;

        // Check operation id doesn't exists
        if let Some(operation_id) = operation_schema.operation_id.as_ref() {
            if self.known_operation_ids.contains(operation_id) {
                return Err(anyhow!("Found duplicate operation_id {operation_id}."));
            }
            self.known_operation_ids.insert(operation_id.clone());
        }

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
            self.add_operation(&path, method, f)?;
        }
        Ok(self)
    }

    /// Generate [`okapi::openapi3::OpenApi`] specification.
    ///
    /// This method can be called repeatedly on the same object.
    pub fn build(&mut self) -> Result<OpenApi, anyhow::Error> {
        let mut spec = self.spec.clone();

        self.operations.sort_by(|lkey, _, rkey, _| {
            let lkey_str = (&lkey.0, lkey.1.as_str());
            let rkey_str = (&rkey.0, rkey.1.as_str());
            lkey_str.cmp(&rkey_str)
        });

        for ((path, method), generator) in &self.operations {
            try_add_path(
                &mut spec,
                &mut self.components,
                &self.builder_options,
                path,
                method.clone(),
                *generator,
            )
            .with_context(|| format!("Failed to add {method} {path}"))?;
        }

        spec.components = Some(self.components.okapi_components()?);

        Ok(spec)
    }

    // Helpers to set OpenApi info/servers/tags/... as is

    /// Set specification title.
    ///
    /// Empty string by default.
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
        self.spec.info.title = title.into();
        self
    }

    /// Set specification version.
    ///
    /// Empty string by default.
    pub fn version(&mut self, version: impl Into<String>) -> &mut Self {
        self.spec.info.version = version.into();
        self
    }

    /// Add description to specification.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.spec.info.description = Some(description.into());
        self
    }

    /// Add contact to specification.
    pub fn contact(&mut self, contact: Contact) -> &mut Self {
        self.spec.info.contact = Some(contact);
        self
    }

    /// Add license to specification.
    pub fn license(&mut self, license: License) -> &mut Self {
        self.spec.info.license = Some(license);
        self
    }

    /// Add terms_of_service to specification.
    pub fn terms_of_service(&mut self, terms_of_service: impl Into<String>) -> &mut Self {
        self.spec.info.terms_of_service = Some(terms_of_service.into());
        self
    }

    /// Add server to specification.
    pub fn server(&mut self, server: Server) -> &mut Self {
        self.spec.servers.push(server);
        self
    }

    /// Add tag to specification.
    pub fn tag(&mut self, tag: Tag) -> &mut Self {
        self.spec.tags.push(tag);
        self
    }

    /// Set external documentation for specification.
    pub fn external_docs(&mut self, docs: ExternalDocs) -> &mut Self {
        let _ = self.spec.external_docs.insert(docs);
        self
    }

    /// Add security scheme definition to specification.
    pub fn security_scheme<N>(&mut self, name: N, sec: SecurityScheme) -> &mut Self
    where
        N: Into<String>,
    {
        self.components.add_security_scheme(name, sec);
        self
    }
}

fn try_add_path(
    spec: &mut OpenApi,
    components: &mut Components,
    builder_options: &BuilderOptions,
    path: &str,
    method: Method,
    generator: OperationGenerator,
) -> Result<(), anyhow::Error> {
    let operation_schema = generator(components, builder_options)?;
    let path_str = path;
    let path = spec.paths.entry(path.into()).or_default();
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
        return Err(anyhow::anyhow!(
            "Unsupported method {method} (at {path_str})"
        ));
    }
    Ok(())
}

/// Ensures that a builder always generates the same file every time, by not relying on
/// internal data structures that may contain random ordering, e.g. [`std::collections::HashMap`].
#[test]
fn ensure_builder_deterministic() {
    use okapi::openapi3::Operation;

    let mut built_specs = Vec::new();

    // generate 100 specs
    for _ in 0..100 {
        let mut builder = OpenApiBuilder::new("title", "version");
        for i in 0..2 {
            builder.operation(format!("/path/{}", i), Method::GET, |_, _| {
                Ok(Operation::default())
            });
        }

        let spec = builder
            .build()
            .map(|x| format!("{:?}", x))
            .expect("Failed to build spec");
        built_specs.push(spec);
    }

    // ensure all specs are the same
    for i in 1..built_specs.len() {
        assert_eq!(built_specs[i - 1], built_specs[i]);
    }
}

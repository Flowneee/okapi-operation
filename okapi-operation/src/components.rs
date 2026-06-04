use okapi::{
    openapi3::{Parameter, ParameterValue, RefOr, SchemaObject, SecurityScheme},
    schemars::{
        JsonSchema,
        gen::{SchemaGenerator, SchemaSettings},
        schema::Schema,
    },
};

/// Builder for [`Components`]
pub struct ComponentsBuilder {
    components: okapi::openapi3::Components,
    inline_subschemas: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for ComponentsBuilder {
    fn default() -> Self {
        Self {
            components: Default::default(),
            inline_subschemas: false,
        }
    }
}

impl ComponentsBuilder {
    pub fn okapi_components(mut self, components: okapi::openapi3::Components) -> Self {
        self.components = components;
        self
    }

    /// Enable or disable subschemas [inlining](https://docs.rs/schemars/latest/schemars/gen/struct.SchemaSettings.html#structfield.inline_subschemas).
    ///
    /// `false` by default.
    pub fn inline_subschemas(mut self, inline_subschemas: bool) -> Self {
        self.inline_subschemas = inline_subschemas;
        self
    }

    pub fn build(self) -> Components {
        let mut generator_settings = SchemaSettings::openapi3();
        generator_settings.inline_subschemas = self.inline_subschemas;
        Components {
            generator: generator_settings.into_generator(),
            components: self.components,
        }
    }
}

/// Storage for reusable components (schemas/parameters/responses/...).
#[derive(Clone)]
pub struct Components {
    generator: SchemaGenerator,
    components: okapi::openapi3::Components,
}

impl Components {
    pub(crate) fn new(components: okapi::openapi3::Components) -> Self {
        ComponentsBuilder::default()
            .okapi_components(components)
            .build()
    }

    /// Get schema for type.
    pub fn schema_for<T: JsonSchema>(&mut self) -> SchemaObject {
        let mut object = self.generator.subschema_for::<T>().into_object();
        for visitor in self.generator.visitors_mut() {
            visitor.visit_schema_object(&mut object);
        }
        object
    }

    /// Expand an axum-style `Path<T>` extractor into OpenAPI path parameters.
    ///
    /// The struct-vs-scalar decision is made here, at runtime, because the
    /// `#[openapi]` macro only sees the syntactic type name `T` and cannot tell
    /// a struct from a primitive at compile time.
    ///
    /// - If `T`'s schema is an object with named fields (a struct), one path
    ///   parameter is produced per field (field name → parameter name, field
    ///   schema → parameter schema). This mirrors how axum deserializes
    ///   `Path<Struct>` by field name.
    /// - Otherwise (scalar / newtype), a single parameter named `fallback_name`
    ///   is produced with `T`'s schema.
    pub fn infer_path_parameters<T: JsonSchema>(&mut self, fallback_name: &str) -> Vec<Parameter> {
        let schema = self.schema_for::<T>();

        // Collect the struct fields, resolving a possible `$ref` against the
        // generator's definitions (schemars references named structs by default).
        // Done in a block so the immutable `definitions()` borrow is released
        // before the `visitors_mut()` call below.
        let properties: Option<Vec<(String, SchemaObject)>> = {
            let resolved = match &schema.reference {
                Some(reference) => reference
                    .rsplit('/')
                    .next()
                    .and_then(|name| self.generator.definitions().get(name))
                    .and_then(|s| match s {
                        Schema::Object(obj) => Some(obj.clone()),
                        Schema::Bool(_) => None,
                    }),
                None => Some(schema.clone()),
            };
            resolved
                .and_then(|obj| obj.object)
                .filter(|obj| !obj.properties.is_empty())
                .map(|obj| {
                    obj.properties
                        .into_iter()
                        .map(|(name, prop)| {
                            let prop_object = match prop {
                                Schema::Object(obj) => obj,
                                Schema::Bool(_) => SchemaObject::default(),
                            };
                            (name, prop_object)
                        })
                        .collect()
                })
        };

        match properties {
            Some(fields) => fields
                .into_iter()
                .map(|(name, mut prop_object)| {
                    // Apply the same visitor passes `schema_for` runs on schemas.
                    for visitor in self.generator.visitors_mut() {
                        visitor.visit_schema_object(&mut prop_object);
                    }
                    path_parameter(name, prop_object)
                })
                .collect(),
            None => vec![path_parameter(fallback_name.to_string(), schema)],
        }
    }

    /// Add security scheme to components.
    pub fn add_security_scheme<N>(&mut self, name: N, sec: SecurityScheme)
    where
        N: Into<String>,
    {
        self.components
            .security_schemes
            .insert(name.into(), RefOr::Object(sec));
    }

    /// Generate [`okapi::openapi3::Components`].
    pub(crate) fn okapi_components(
        &mut self,
    ) -> Result<okapi::openapi3::Components, anyhow::Error> {
        let mut components = self.components.clone();
        for (name, mut schema_object) in self
            .generator
            .definitions()
            .iter()
            .map(|(n, s)| (n.clone(), s.clone().into_object()))
            .collect::<Vec<_>>()
        {
            for visitor in self.generator.visitors_mut() {
                visitor.visit_schema_object(&mut schema_object);
            }
            if components.schemas.contains_key(&name) {
                return Err(anyhow::anyhow!("Multiple schemas found for '{}'", name));
            }
            let _ = components.schemas.insert(name, schema_object);
        }
        Ok(components)
    }
}

/// Build a required `path` parameter from a name and schema. Mirrors the literal
/// produced by the macro in `okapi-operation-macro`'s `path::Path::to_tokens`.
fn path_parameter(name: String, schema: SchemaObject) -> Parameter {
    Parameter {
        name,
        location: "path".into(),
        description: None,
        required: true,
        deprecated: false,
        allow_empty_value: false,
        value: ParameterValue::Schema {
            style: None,
            explode: None,
            allow_reserved: false,
            schema,
            example: Default::default(),
            examples: Default::default(),
        },
        extensions: Default::default(),
    }
}

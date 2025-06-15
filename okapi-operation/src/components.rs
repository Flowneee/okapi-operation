use okapi::{
    openapi3::{RefOr, SchemaObject, SecurityScheme},
    schemars::{
        JsonSchema,
        gen::{SchemaGenerator, SchemaSettings},
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

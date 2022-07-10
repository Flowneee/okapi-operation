use okapi::{
    openapi3::{RefOr, SchemaObject, SecurityScheme},
    schemars::{
        gen::{SchemaGenerator, SchemaSettings},
        JsonSchema,
    },
};

/// Storage for reusable components (schemas/parameters/responses/...).
pub struct Components {
    generator: SchemaGenerator,
    components: okapi::openapi3::Components,
}

impl Components {
    pub(crate) fn new(components: okapi::openapi3::Components) -> Self {
        Self {
            generator: SchemaSettings::openapi3().into_generator(),
            components,
        }
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
    pub fn add_security<N>(&mut self, name: N, sec: SecurityScheme)
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

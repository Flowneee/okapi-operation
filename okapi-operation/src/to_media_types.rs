use okapi::{openapi3::MediaType, Map};

use crate::Components;

/// Generate [`MediaType`] for type.
pub trait ToMediaTypes {
    fn generate(components: &mut Components) -> Result<Map<String, MediaType>, anyhow::Error>;
}

/// Generate [`ToMediaTypes`] implementation for newtype.
///
/// Inner type should implement `schemars::JsonSchema`.
///
/// # Example
///
/// ```rust,compile
/// # use okapi_operation::*;
/// struct JsonWrapper<T>(T);
///
/// impl_to_media_types_for_wrapper!(JsonWrapper<T>, "application/json");
/// ```
#[macro_export]
macro_rules! impl_to_media_types_for_wrapper {
    ($ty:path, $mime:literal) => {
        impl<T: $crate::schemars::JsonSchema> $crate::ToMediaTypes for $ty {
            fn generate(
                components: &mut $crate::Components,
            ) -> Result<
                    $crate::okapi::Map<String, $crate::okapi::openapi3::MediaType>,
                    $crate::anyhow::Error
                >
            {
                let schema = components.schema_for::<T>();
                Ok($crate::okapi::map! {
                    $mime.into() => {
                        $crate::okapi::openapi3::MediaType { schema: Some(schema), ..Default::default() }
                    }
                })
            }
        }
    };
}

impl ToMediaTypes for () {
    fn generate(_components: &mut Components) -> Result<Map<String, MediaType>, anyhow::Error> {
        Ok(okapi::map! {})
    }
}

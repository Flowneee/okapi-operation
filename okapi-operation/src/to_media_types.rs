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
    ($ty:path, $mime:expr) => {
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

macro_rules! forward_impl_to_media_types {
    ($ty_for:ty, $ty_base:ty) => {
        impl $crate::ToMediaTypes for $ty_for {
            fn generate(
                components: &mut $crate::Components,
            ) -> Result<
                $crate::okapi::Map<String, $crate::okapi::openapi3::MediaType>,
                $crate::anyhow::Error,
            > {
                <$ty_base as $crate::ToMediaTypes>::generate(components)
            }
        }
    };
}

mod impls {
    use std::borrow::Cow;

    use bytes::{Bytes, BytesMut};
    use mime::{APPLICATION_OCTET_STREAM, TEXT_PLAIN};
    use okapi::{
        map,
        openapi3::SchemaObject,
        schemars::schema::{InstanceType, SingleOrVec},
    };

    use super::*;

    impl ToMediaTypes for () {
        fn generate(_components: &mut Components) -> Result<Map<String, MediaType>, anyhow::Error> {
            Ok(map! {})
        }
    }

    impl ToMediaTypes for String {
        fn generate(_components: &mut Components) -> Result<Map<String, MediaType>, anyhow::Error> {
            Ok(map! {
                TEXT_PLAIN.to_string() => MediaType::default()
            })
        }
    }
    forward_impl_to_media_types!(&'static str, String);
    forward_impl_to_media_types!(Cow<'static, str>, String);

    impl ToMediaTypes for Vec<u8> {
        fn generate(_components: &mut Components) -> Result<Map<String, MediaType>, anyhow::Error> {
            // In schemars Bytes defined as array of integers, but OpenAPI recommend
            // use string type with binary format
            // https://swagger.io/docs/specification/describing-request-body/file-upload/
            let schema = SchemaObject {
                instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                format: Some("binary".into()),
                ..SchemaObject::default()
            };

            Ok(map! {
                APPLICATION_OCTET_STREAM.to_string() => MediaType {
                    schema: Some(schema),
                    ..MediaType::default()
                },
            })
        }
    }
    forward_impl_to_media_types!(&'static [u8], Vec<u8>);
    forward_impl_to_media_types!(Cow<'static, [u8]>, Vec<u8>);
    forward_impl_to_media_types!(Bytes, Vec<u8>);
    forward_impl_to_media_types!(BytesMut, Vec<u8>);
}

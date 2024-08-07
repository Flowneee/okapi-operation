use okapi::openapi3::{RefOr, Responses};

use crate::Components;

/// Generate [`Responses`] for type.
pub trait ToResponses {
    fn generate(components: &mut Components) -> Result<Responses, anyhow::Error>;
}

/// Generate [`ToResponses`] implementation for newtype.
///
/// Inner type should implement `ToMediaTypes`.
///
/// # Example
///
/// ```rust,compile
/// # use speka::*;
/// # impl_to_media_types_for_wrapper!(JsonWrapper<T>, "application/json");
/// struct JsonWrapper<T>(T);
///
/// impl_to_responses_for_wrapper!(JsonWrapper<T>);
/// ```
#[macro_export]
macro_rules! impl_to_responses_for_wrapper {
    ($ty:path) => {
        impl<T: $crate::schemars::JsonSchema> $crate::ToResponses for $ty {
            fn generate(components: &mut $crate::Components) -> Result<$crate::okapi::openapi3::Responses, $crate::anyhow::Error> {
                let media_types = <$ty as $crate::ToMediaTypes>::generate(components)?;
                Ok($crate::okapi::openapi3::Responses {
                    responses: $crate::okapi::map! {
                        "200".into() => $crate::okapi::openapi3::RefOr::Object(
                            $crate::okapi::openapi3::Response { content: media_types, ..Default::default() }
                        )
                    },
                    ..Default::default()
                })
            }
        }
    };
}

macro_rules! forward_impl_to_responses {
    ($ty_for:ty, $ty_base:ty) => {
        impl $crate::ToResponses for $ty_for {
            fn generate(
                components: &mut $crate::Components,
            ) -> Result<$crate::okapi::openapi3::Responses, $crate::anyhow::Error> {
                <$ty_base as $crate::ToResponses>::generate(components)
            }
        }
    };
}

mod impls {
    use std::borrow::Cow;

    use bytes::{Bytes, BytesMut};
    use okapi::openapi3::Response;

    use super::*;
    use crate::ToMediaTypes;

    impl ToResponses for () {
        fn generate(_components: &mut Components) -> Result<Responses, anyhow::Error> {
            Ok(Responses {
                responses: okapi::map! {
                    "200".into() => RefOr::Object(Default::default())
                },
                ..Default::default()
            })
        }
    }

    impl<T, E> ToResponses for Result<T, E>
    where
        T: ToResponses,
        E: ToResponses,
    {
        fn generate(components: &mut Components) -> Result<Responses, anyhow::Error> {
            let overlap_err_fn = |status| {
                anyhow::anyhow!(
                    "Type {} produces {} response in both Ok and Err variants",
                    std::any::type_name::<Self>(),
                    status
                )
            };
            let mut ok = T::generate(components)?;
            let err = E::generate(components)?;

            if ok.default.is_some() && err.default.is_some() {
                return Err(overlap_err_fn("default"));
            }
            ok.default = ok.default.or(err.default);

            for (status, response) in err.responses.into_iter() {
                if ok.responses.contains_key(&status) {
                    return Err(overlap_err_fn(&status));
                }
                let _ = ok.responses.insert(status, response);
            }

            Ok(ok)
        }
    }

    impl ToResponses for String {
        fn generate(components: &mut Components) -> Result<Responses, anyhow::Error> {
            Ok(Responses {
                responses: okapi::map! {
                    "200".into() => RefOr::Object(Response {
                        content: <Self as ToMediaTypes>::generate(components)?,
                        ..Default::default()
                    })
                },
                ..Default::default()
            })
        }
    }
    forward_impl_to_responses!(&'static str, String);
    forward_impl_to_responses!(Cow<'static, str>, String);

    impl ToResponses for Vec<u8> {
        fn generate(components: &mut Components) -> Result<Responses, anyhow::Error> {
            Ok(Responses {
                responses: okapi::map! {
                    "200".into() => RefOr::Object(Response {
                        content: <Self as ToMediaTypes>::generate(components)?,
                        ..Default::default()
                    })
                },
                ..Default::default()
            })
        }
    }
    forward_impl_to_responses!(&'static [u8], Vec<u8>);
    forward_impl_to_responses!(Cow<'static, [u8]>, Vec<u8>);
    forward_impl_to_responses!(Bytes, Vec<u8>);
    forward_impl_to_responses!(BytesMut, Vec<u8>);
}

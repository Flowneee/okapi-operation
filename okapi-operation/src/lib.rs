#![allow(rustdoc::bare_urls)]
#![doc = include_str!("../docs/root.md")]

pub use anyhow;
#[doc(no_inline)]
pub use okapi::{
    self,
    openapi3::OpenApi,
    schemars::{self, JsonSchema},
};
#[cfg(feature = "macro")]
#[doc(inline)]
pub use okapi_operation_macro::openapi;

#[cfg(feature = "axum-integration")]
pub mod axum_integration;

pub use self::{
    builder::OpenApiBuilder,
    components::{Components, ComponentsBuilder},
    to_media_types::ToMediaTypes,
    to_responses::ToResponses,
};

use okapi::openapi3::Operation;

mod builder;
mod components;
mod to_media_types;
mod to_responses;
mod utils;

/// Empty type alias (for using in attribute values).
pub type Empty = ();

// TODO: allow return RefOr<Operation>
/// Operation generator signature.
pub type OperationGenerator = fn(&mut Components) -> Result<Operation, anyhow::Error>;

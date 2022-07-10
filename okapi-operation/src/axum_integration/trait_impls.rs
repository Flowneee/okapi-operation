use crate::{impl_to_media_types_for_wrapper, impl_to_responses_for_wrapper};

impl_to_media_types_for_wrapper!(axum::Json<T>, "application/json");

impl_to_responses_for_wrapper!(axum::Json<T>);

use std::{collections::HashSet, sync::LazyLock};

use syn::{PatType, Type};

use super::{RequestBody, RequestBodyAttrs};
use crate::error::Error;

// NOTE: `Form` is not enabled because it have different content types
// based on method https://docs.rs/axum/latest/axum/struct.Form.html#as-extractor
static KNOWN_BODY_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "String", // std types
        "Json",   // 3rd party types
        "Bytes",  // 3rd party types
    ]
    .into_iter()
    .collect()
});

impl RequestBody {
    pub(super) fn try_find_axum(pt: &PatType) -> Result<Option<Self>, Error> {
        let Type::Path(ref path) = *pt.ty else {
            return Ok(None);
        };
        for pat_seg in path.path.segments.iter().rev() {
            if KNOWN_BODY_TYPES.contains(pat_seg.ident.to_string().as_str()) {
                return Ok(Some(Self {
                    argument_type: *pt.ty.clone(),
                    attrs: RequestBodyAttrs::default(),
                }));
            }
        }

        Ok(None)
    }
}

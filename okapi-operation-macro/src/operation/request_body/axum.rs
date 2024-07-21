use std::collections::HashSet;

use syn::{PatType, Type};

use super::{RequestBody, RequestBodyAttrs};
use crate::error::Error;

lazy_static::lazy_static! {
    static ref KNOWN_BODY_TYPES: HashSet<&'static str> = [
        "Json",
    ].into_iter().collect();
}

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

#![allow(clippy::manual_unwrap_or_default)]

use syn::{parse_macro_input, ItemFn};

mod error;
mod operation;
mod utils;

static OPENAPI_FUNCTION_NAME_SUFFIX: &str = "__openapi";

#[proc_macro_attribute]
pub fn openapi(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match operation::openapi(attr, parse_macro_input!(input as ItemFn)) {
        Ok(x) => x.into(),
        Err(err) => err.write().into(),
    }
}

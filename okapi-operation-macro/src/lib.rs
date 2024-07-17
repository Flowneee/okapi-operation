#![allow(clippy::manual_unwrap_or_default)]

use syn::{parse_macro_input, AttributeArgs, ItemFn};

mod error;
// mod handler;
mod operation;
mod utils;

static OPENAPI_FUNCTION_NAME_SUFFIX: &str = "__openapi";

#[proc_macro_attribute]
pub fn openapi(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match operation::openapi(
        parse_macro_input!(attr as AttributeArgs),
        parse_macro_input!(input as ItemFn),
    ) {
        Ok(x) => x.into(),
        Err(err) => err.write().into(),
    }
}

// /// Macro for expanding and binding OpenAPI operation specification
// /// generator to handler or service.
// #[proc_macro]
// pub fn openapi_handler(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     match handler::openapi_handler(parse_macro_input!(input as Path)) {
//         Ok(x) => x.into(),
//         Err(err) => err.write().into(),
//     }
// }

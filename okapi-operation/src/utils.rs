/// Convert Axum path with templates to OpenAPI format.
pub(crate) fn convert_axum_path_to_openapi(path: &str) -> String {
    path.split('/')
        .map(|x| {
            if x.starts_with(':') {
                format!("{{{}}}", x.trim_matches(':'))
            } else {
                x.into()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

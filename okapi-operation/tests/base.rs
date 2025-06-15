use okapi_operation::openapi;

#[test]
#[allow(unused)]
fn crate_name_override() {
    use okapi_operation as renamed_crate;

    #[openapi(crate = "renamed_crate")]
    async fn handle() {}
}

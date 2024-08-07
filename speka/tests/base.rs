use speka::openapi;

#[test]
#[allow(unused)]
fn crate_name_override() {
    use speka as renamed_crate;

    #[openapi(crate = "renamed_crate")]
    async fn handle() {}
}

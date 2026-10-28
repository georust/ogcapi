#![cfg(all(
    feature = "common",
    feature = "edr",
    feature = "features",
    feature = "processes",
    feature = "stac",
    feature = "styles",
    feature = "tiles",
))]

mod setup;

#[tokio::test]
async fn openapi_definition() -> anyhow::Result<()> {
    use ogcapi_client::Client;

    let (addr, _) = setup::spawn_app().await?;
    let client = Client::new(&format!("http://{addr}"))?;
    let api: serde_json::Value = client.fetch(&format!("http://{addr}/api_v3.1")).await?;

    insta::with_settings!({ sort_maps => true }, {
        insta::assert_json_snapshot!(api);
    });
    Ok(())
}

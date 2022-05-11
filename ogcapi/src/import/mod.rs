mod args;
mod boundaries;
pub mod geojson;
pub mod ogr;
pub mod osm;

pub use args::Args;

#[cfg(feature = "stac")]
pub(crate) async fn load_asset_from_path(
    path: &std::path::PathBuf,
) -> anyhow::Result<std::collections::HashMap<String, ogcapi_types::stac::Asset>> {
    use ogcapi_drivers::s3::{ByteStream, S3};
    use ogcapi_types::common::media_type::GEO_JSON;

    // Setup S3 driver
    let s3 = S3::setup().await;

    let stream = ByteStream::from_path(&path).await?;

    // Upload asset
    let filename = path.file_name().unwrap().to_str().unwrap();

    let key = format!("assets/{filename}");

    s3.client
        .put_object()
        .bucket("test-bucket")
        .key(&key)
        .body(stream)
        .content_type(GEO_JSON)
        .send()
        .await?;

    let asset = ogcapi_types::stac::Asset::new(key);

    let file_stem = path.file_stem().unwrap().to_str().unwrap();

    Ok(std::collections::HashMap::from([(
        file_stem.to_string(),
        asset,
    )]))
}

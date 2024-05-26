use std::collections::HashMap;

use ogcapi::{
    drivers::s3::{ByteStream, S3},
    types::{common::media_type::GEO_JSON, stac::Asset},
};

pub async fn load_asset_from_path(
    path: &std::path::PathBuf,
) -> anyhow::Result<HashMap<String, Asset>> {
    // Setup S3 driver
    let s3 = S3::new().await;

    let stream = ByteStream::from_path(&path).await?;

    // Upload asset
    let filename = path.file_name().unwrap().to_str().unwrap();

    let key = format!("assets/{filename}");
    let bucket = std::env::var("AWS_S3_BUCKET_NAME")?;

    s3.client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .body(stream)
        .content_type(GEO_JSON)
        .send()
        .await?;

    let asset = Asset::new(key);

    let file_stem = path.file_stem().unwrap().to_str().unwrap();

    Ok(HashMap::from([(file_stem.to_string(), asset)]))
}

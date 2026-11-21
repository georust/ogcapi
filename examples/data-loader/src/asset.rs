//! The input as a STAC asset of its collection, in an object store.

use std::{collections::HashMap, path::Path};

use anyhow::Context;
use url::Url;

use ogcapi::{
    drivers::object::ObjectDriver,
    types::{common::media_type::GEO_JSON, stac::Asset},
};

/// Uploads the input file to the object store at `store` as an asset of the
/// collection, named by its file, and lists it among the collection's assets
/// at `target`: the API at an `http(s)` URL through the client, or the
/// database at a `postgres` URL.
pub async fn attach(
    input: &Path,
    collection: &str,
    store: &Url,
    target: &Url,
) -> anyhow::Result<()> {
    let name = input
        .file_name()
        .and_then(|name| name.to_str())
        .context("the input names no file")?;
    let media_type = crate::is_geojson_file(input).then_some(GEO_JSON);

    let location = ObjectDriver::from_url(store)?
        .put_asset(collection, name, std::fs::read(input)?, media_type)
        .await?;
    let mut asset = Asset::new(format!(
        "{}/{location}",
        store.as_str().trim_end_matches('/')
    ));
    asset.r#type = media_type.map(str::to_owned);
    let key = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name)
        .to_owned();
    let assets = HashMap::from([(key, asset)]);
    tracing::info!("Uploaded {name} to {store}");

    if target.scheme().starts_with("http") {
        #[cfg(feature = "client")]
        {
            let client = ogcapi::client::Client::new(target.as_str())?;
            let mut collection = client.collection(collection).await?;
            collection.assets.extend(assets);
            client.update_collection(&collection).await?;
            return Ok(());
        }
    } else if target.scheme().starts_with("postgres") {
        #[cfg(feature = "postgres")]
        {
            use ogcapi::drivers::CollectionTransactions;

            let db = ogcapi::drivers::postgres::Db::setup(target).await?;
            let mut collection = db
                .read_collection(collection)
                .await?
                .context("no collection to list the asset in")?;
            collection.assets.extend(assets);
            db.update_collection(&collection).await?;
            return Ok(());
        }
    }
    anyhow::bail!("the collection at {target} can't take the asset here")
}

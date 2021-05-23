mod boundaries;
mod gdal;
mod osm;

use sqlx::{postgres::PgPoolOptions, types::Json, Pool, Postgres};

use crate::{
    collections::{Collection, Extent, ItemType, Provider, Summaries},
    common::Link,
};

pub async fn import(
    input: std::path::PathBuf,
    filter: &Option<String>,
    collection: &Option<String>,
) -> Result<(), anyhow::Error> {
    // Create a connection pool
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Import data
    if input.extension() == Some(std::ffi::OsStr::new("pbf")) {
        osm::osm_import(input, &filter, &collection, &pool).await
    } else {
        gdal::gdal_import(input, &filter, &collection, &pool).await
    }
}

/// Insert collection metadata
async fn insert_collection(
    collection: &Collection,
    pool: &Pool<Postgres>,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query_file!(
        "sql/collection_insert.sql",
        collection.id,
        collection.title,
        collection.description,
        collection.links as _,
        collection.extent as _,
        collection.item_type as _,
        collection.crs.as_deref(),
        collection.storage_crs,
        collection.storage_crs_coordinate_epoch,
        collection.stac_version,
        collection.stac_extensions.as_deref(),
        collection.keywords.as_deref(),
        collection.licence,
        collection.providers as _,
        collection.summaries as _
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}

/// Delete collection metadata
async fn delete_collection(id: &str, pool: &Pool<Postgres>) -> Result<(), anyhow::Error> {
    let _ = sqlx::query_file!("sql/collection_delete.sql", id)
        .fetch_optional(pool)
        .await?;
    Ok(())
}

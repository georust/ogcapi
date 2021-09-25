mod boundaries;
mod gdal;
mod osm;

use std::env;

use crate::db::Db;

pub async fn import(
    input: std::path::PathBuf,
    filter: &Option<String>,
    collection: &Option<String>,
    s_srs: &Option<u32>,
    t_srs: &Option<u32>,
) -> Result<(), anyhow::Error> {
    // Setup a db connection pool
    let db = Db::connect(&env::var("DATABASE_URL")?).await?;

    // Import data
    if input.extension() == Some(std::ffi::OsStr::new("pbf")) {
        osm::osm_import(input, &filter, &collection, &db).await
    } else {
        gdal::gdal_import(input, &filter, &collection, &s_srs, &t_srs, &db).await
    }
}

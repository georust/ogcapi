#[cfg(feature = "stac")]
use std::{collections::HashMap, path::PathBuf};

use geojson::GeoJson;
use serde_json::Value;

use ogcapi_drivers::{postgres::Db, CollectionTransactions};
use ogcapi_types::common::{Collection, Crs};

use super::Args;

pub async fn load(args: Args, show_pb: bool) -> anyhow::Result<()> {
    // Setup drivers
    let db = Db::setup(&args.database_url).await?;

    // Create collection
    let collection = Collection {
        id: args.collection.to_owned(),
        item_type: Some("Feature".to_string()),
        crs: vec![
            Crs::default(),
            Crs::from(4326),
            Crs::from(3857),
            Crs::from(2056),
        ],
        storage_crs: Some(Crs::from(4326)),
        #[cfg(feature = "stac")]
        assets: load_asset_from_path(&args.input).await?,
        ..Default::default()
    };

    db.delete_collection(&collection.id).await?;
    db.create_collection(&collection).await?;

    // Load features
    let geojson_str = std::fs::read_to_string(&args.input)?;
    let geojson = geojson_str.parse::<GeoJson>()?;

    match geojson {
        GeoJson::FeatureCollection(mut fc) => {
            let mut tx = db.pool.begin().await?;

            let mut pb = pbr::ProgressBar::new(fc.features.len() as u64);

            for (i, feature) in fc.features.iter_mut().enumerate() {
                let id = if let Some(id) = &feature.id {
                    match id {
                        geojson::feature::Id::String(id) => id.to_owned(),
                        geojson::feature::Id::Number(id) => id.to_string(),
                    }
                } else {
                    i.to_string()
                };

                sqlx::query(&format!(
                    r#"
                    INSERT INTO items.{} (
                        id,
                        type,
                        properties,
                        geom
                    ) VALUES ($1, 'Feature', $2, ST_GeomFromGeoJSON($3))
                "#,
                    collection.id
                ))
                .bind(id)
                .bind(Value::from(feature.properties.take().unwrap()))
                .bind(feature.geometry.take().unwrap().value.to_string())
                .execute(&mut tx)
                .await?;

                if show_pb {
                    pb.inc();
                }
            }
            pb.finish_println("");

            tx.commit().await?;
        }
        GeoJson::Geometry(_) | GeoJson::Feature(_) => todo!(),
    }

    Ok(())
}

#[cfg(feature = "stac")]
pub async fn load_asset_from_path(
    path: &PathBuf,
) -> anyhow::Result<HashMap<String, ogcapi_types::stac::Asset>> {
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

    Ok(HashMap::from([(file_stem.to_string(), asset)]))
}

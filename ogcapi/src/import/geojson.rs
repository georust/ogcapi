use geojson::GeoJson;
use serde_json::Value;

use ogcapi_drivers::{postgres::Db, CollectionTransactions};
use ogcapi_types::common::{Collection, Crs, Extent, SpatialExtent};

use super::Args;

pub async fn load(args: Args, show_pb: bool) -> anyhow::Result<()> {
    // Setup driver
    let db = Db::new().await?;

    // Load data
    let geojson_str = std::fs::read_to_string(&args.input)?;
    let geojson = geojson_str.parse::<GeoJson>()?;

    match geojson {
        GeoJson::FeatureCollection(mut fc) => {
            let mut tx = db.pool.begin().await?;

            // Create collection
            let collection = Collection {
                id: args.collection.to_owned(),
                item_type: Some("Feature".to_string()),
                extent: fc
                    .bbox
                    .map(|bbox| Extent {
                        spatial: Some(SpatialExtent {
                            bbox: vec![bbox
                                .as_slice()
                                .try_into()
                                .unwrap_or_else(|_| [-180.0, -90.0, 180.0, 90.0].into())],
                            crs: Crs::default(),
                        }),
                        ..Default::default()
                    })
                    .or_else(|| Some(Extent::default())),
                crs: vec![Crs::default(), Crs::from_epsg(3857), Crs::from_epsg(2056)],
                storage_crs: Some(Crs::default()),
                #[cfg(feature = "stac")]
                assets: crate::import::load_asset_from_path(&args.input).await?,
                ..Default::default()
            };

            db.delete_collection(&collection.id).await?;
            db.create_collection(&collection).await?;

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
                        properties,
                        geom
                    ) VALUES ($1, $2, ST_GeomFromGeoJSON($3))
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

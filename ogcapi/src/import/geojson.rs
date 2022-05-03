use geojson::GeoJson;
use serde_json::Value;

use ogcapi_drivers::postgres::Db;
use ogcapi_types::common::{Collection, Crs};

use super::Args;

pub async fn load(args: Args, show_pb: bool) -> anyhow::Result<()> {
    // Setup a db connection pool
    let db = Db::setup(&args.database_url).await?;

    // Create colection
    let collection = Collection {
        id: args.collection.to_owned(),
        item_type: Some("Feature".to_string()),
        crs: vec![Crs::default(), Crs::from(4326), Crs::from(3857)],
        storage_crs: Some(Crs::from(4326)),
        ..Default::default()
    };

    db.delete_collection(&collection.id).await?;
    db.insert_collection(&collection).await?;

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

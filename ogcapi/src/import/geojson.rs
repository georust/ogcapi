use std::convert::TryInto;

use geo::Geometry;

use ogcapi_drivers::{postgres::Db, CollectionTransactions};
use ogcapi_types::common::{Collection, Crs, Extent, SpatialExtent};

use super::Args;

pub async fn load(args: Args) -> anyhow::Result<()> {
    // Setup driver
    let db = Db::new().await?;

    // Extract data
    let geojson_str = std::fs::read_to_string(&args.input)?;
    let geojson = geojson_str.parse::<geojson::FeatureCollection>()?;

    // Create collection
    let collection = Collection {
        id: args.collection.to_owned(),
        item_type: Some("Feature".to_string()),
        extent: geojson
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

    // Load features
    let now = std::time::Instant::now();
    let count = geojson.features.len();

    bulk_load_features(&collection.id, &geojson.features, &db.pool).await?;

    // stats
    let elapsed = now.elapsed().as_millis() as f64 / 1000.0;
    tracing::info!(
        "Loaded {count} features in {elapsed} seconds ({:.2}/s)",
        count as f64 / elapsed
    );

    Ok(())
}

async fn bulk_load_features(
    collection: &str,
    features: &[geojson::Feature],
    pool: &sqlx::PgPool,
) -> anyhow::Result<()> {
    let mut ids = Vec::new();
    let mut properties = Vec::new();
    let mut geoms = Vec::new();

    for (i, feature) in features.iter().enumerate() {
        let id = if let Some(id) = &feature.id {
            match id {
                geojson::feature::Id::String(id) => id.to_owned(),
                geojson::feature::Id::Number(id) => id.to_string(),
            }
        } else {
            i.to_string()
        };
        ids.push(id);

        properties.push(feature.properties.to_owned().map(sqlx::types::Json));

        let geom = Geometry::try_from(feature.geometry.to_owned().unwrap().value).unwrap();
        geoms.push(wkb::geom_to_wkb::<f64>(&geom).unwrap());
    }

    super::bulk_load_items(collection, &ids, &properties[..], &geoms[..], pool).await?;

    Ok(())
}

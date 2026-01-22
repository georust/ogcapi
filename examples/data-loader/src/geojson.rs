use std::{convert::TryInto, io::Cursor, time::Instant};

use geo::Geometry;
use geojson::{FeatureCollection, feature::Id};
use sqlx::types::Json;
use wkb::{Endianness, writer::WriteOptions};

use ogcapi::{
    drivers::{CollectionTransactions, postgres::Db},
    types::common::{Collection, Crs, Extent, SpatialExtent},
};

use super::Args;

pub async fn load(args: Args) -> anyhow::Result<()> {
    let now = Instant::now();

    // Setup driver
    let db = Db::setup(&args.database_url).await?;

    // Extract data
    let geojson_str = std::fs::read_to_string(&args.input)?;
    let geojson = geojson_str.parse::<FeatureCollection>()?;

    // Create collection
    let collection = Collection {
        id: args.collection.to_owned(),
        extent: geojson
            .bbox
            .map(|bbox| Extent {
                spatial: SpatialExtent {
                    bbox: vec![
                        bbox.as_slice()
                            .try_into()
                            .unwrap_or_else(|_| [-180.0, -90.0, 180.0, 90.0].into()),
                    ],
                    crs: Some(Crs::default2d()),
                },
                ..Default::default()
            })
            .or_else(|| Some(Extent::default())),
        crs: vec![Crs::default2d(), Crs::from_epsg(3857)],
        storage_crs: Some(Crs::default2d()),
        #[cfg(feature = "stac")]
        assets: crate::asset::load_asset_from_path(&args.input).await?,
        ..Default::default()
    };

    db.delete_collection(&collection.id).await?;
    db.create_collection(&collection).await?;

    // Load features
    let count = geojson.features.len();

    let mut ids = Vec::with_capacity(count);
    let mut properties = Vec::with_capacity(count);
    let mut geoms = Vec::with_capacity(count);

    for (i, feature) in geojson.features.iter().enumerate() {
        // id
        let id = if let Some(id) = &feature.id {
            match id {
                Id::String(id) => id.to_owned(),
                Id::Number(id) => id.to_string(),
            }
        } else {
            i.to_string()
        };
        ids.push(id);

        // properties
        properties.push(feature.properties.to_owned().map(Json));

        // geometry
        let geom = Geometry::try_from(feature.geometry.to_owned().unwrap().value)?;
        let mut wkb = Cursor::new(Vec::new());
        wkb::writer::write_geometry(
            &mut wkb,
            &geom,
            &WriteOptions {
                endianness: Endianness::LittleEndian,
            },
        )
        .unwrap();
        geoms.push(wkb.into_inner());
    }

    sqlx::query(&format!(
        r#"
        INSERT INTO items."{}" (id, properties, geom)
        SELECT * FROM UNNEST($1::text[], $2::jsonb[], $3::bytea[])
        "#,
        collection.id
    ))
    .bind(ids)
    .bind(properties)
    .bind(geoms)
    .execute(&db.pool)
    .await?;

    // stats
    let elapsed = now.elapsed().as_millis() as f64 / 1000.0;
    tracing::info!(
        "Loaded {count} features in {elapsed} seconds ({:.2}/s)",
        count as f64 / elapsed
    );

    Ok(())
}

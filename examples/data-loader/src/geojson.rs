use geojson::FeatureCollection;

use ogcapi::types::common::{Collection, Crs, Extent, SpatialExtent};

fn collection_from_geojson(
    collection_id: &str,
    geojson: &FeatureCollection,
    s_srs: Option<u32>,
) -> Result<Collection, anyhow::Error> {
    // handle crs
    let mut storage_crs = Crs::default2d();

    match s_srs {
        Some(srs) => {
            storage_crs = Crs::from_srid(srs as i32);
        }
        None => {
            // Try parsing named crs
            // TODO: proper crs handling
            let name = geojson
                .foreign_members
                .as_ref()
                .and_then(|attributes| attributes.get("crs").and_then(|v| v.as_object()))
                .and_then(|crs_object| crs_object.get("properties").and_then(|p| p.get("name")));

            if let Some(name) = name {
                storage_crs = serde_json::from_value(name.to_owned())?;
            };
        }
    }

    tracing::debug!("storage_crs: {storage_crs:?}");

    let mut crs = vec![storage_crs.clone(), Crs::default2d(), Crs::from_epsg(3857)];
    crs.sort();
    crs.dedup();

    // handle bbox
    let bbox = match &geojson.bbox {
        Some(bbox) => bbox.as_slice().try_into().ok(),
        None => None,
    };

    let extent = bbox
        .map(|bbox| SpatialExtent {
            bbox: vec![bbox],
            crs: Some(storage_crs.clone()),
        })
        .map(|spatial| Extent {
            spatial: Some(spatial),
            ..Default::default()
        });
    tracing::debug!("extent: {extent:#?}");

    let mut collection = Collection::new(collection_id);
    collection.extent = extent;
    collection.crs = crs;
    collection.storage_crs = Some(storage_crs);

    Ok(collection)
}

#[cfg(feature = "client")]
pub mod client {

    use std::{path::Path, sync::Arc, time::Instant};

    use futures::{StreamExt, TryStreamExt};
    use geojson::FeatureCollection;
    use tokio::sync::RwLock;
    use url::Url;

    use ogcapi::{
        client::{Client, Error},
        types::common::Bbox,
    };

    use super::*;

    /// Load GeoJson collection with [Client].
    pub async fn load(
        input: impl AsRef<Path>,
        collection_id: &str,
        s_srs: Option<u32>,
        public_url: &Url,
    ) -> anyhow::Result<()> {
        let now = Instant::now();

        // Setup client
        let client = Client::new(public_url)?;

        // Extract data
        let geojson_str = std::fs::read_to_string(input)?;
        let geojson = geojson_str.parse::<FeatureCollection>()?;

        // Create collection
        let collection = collection_from_geojson(collection_id, &geojson, s_srs)?;
        client.delete_collection(&collection.id).await?;
        client.create_collection(&collection).await?;

        // Load features
        let count = geojson.features.len();

        let client = Arc::new(client);

        let stream = futures::stream::iter(geojson.features);

        let bbox = Arc::new(RwLock::new(Bbox::new_empty_3d()));

        stream
            .map(Ok::<geojson::Feature, Error>)
            .try_for_each_concurrent(
                std::thread::available_parallelism().ok().map(|t| t.into()),
                |feature| {
                    let client = client.clone();
                    let bbox = bbox.clone();
                    async move {
                        // extract extent
                        if let Some(coords) = feature
                            .bbox
                            .as_ref()
                            .or(feature.geometry.as_ref().and_then(|f| f.bbox.as_ref()))
                        {
                            bbox.write().await.extend_point(coords);
                        } else if let Some(geometry) = &feature.geometry {
                            for position in ogcapi::types::features::coords_iter(geometry) {
                                bbox.write().await.extend_point(position.as_slice());
                            }
                        }

                        let value = serde_json::to_value(feature)?;
                        let feature = serde_json::from_value(value)?;
                        client.create_item(collection_id, &feature).await?;
                        Ok(())
                    }
                },
            )
            .await?;

        // update bbox
        if collection.extent.is_none() {
            let mut bbox = *bbox.write().await;
            if bbox.interval(3).is_empty() {
                bbox = bbox.as_2d()
            };

            tracing::info!("Set spatial extent: {:#?}", &bbox);
            let mut collection = client.collection(collection_id).await?;
            collection.extent = Some(Extent {
                spatial: Some(SpatialExtent {
                    bbox: vec![bbox],
                    crs: collection.storage_crs.clone(),
                }),
                temporal: None,
            });
            client.update_collection(&collection).await?;
        }

        // stats
        let elapsed = now.elapsed().as_secs_f64();
        tracing::info!(
            "Loaded {count} features in {elapsed:.2} seconds ({:.2}/s)",
            count as f64 / elapsed
        );

        Ok(())
    }
}

#[cfg(feature = "postgres")]
pub mod db {

    use std::{path::Path, time::Instant};

    use geojson::{FeatureCollection, feature::Id};
    use serde_json::{Map, Value};
    use sqlx::{Pool, Postgres, postgres::PgQueryResult, types::Json};
    use url::Url;

    use ogcapi::drivers::{CollectionTransactions, postgres::Db};

    use super::*;

    /// Bulk load GeoJson collection to [Db].
    pub async fn load(
        input: impl AsRef<Path>,
        collection_id: &str,
        s_srs: Option<u32>,
        database_url: &Url,
    ) -> anyhow::Result<()> {
        let now = Instant::now();

        // Setup driver
        let db = Db::setup(database_url).await?;

        // Extract data
        let geojson_str = std::fs::read_to_string(input)?;
        let geojson = geojson_str.parse::<FeatureCollection>()?;

        // Create collection
        let collection = collection_from_geojson(collection_id, &geojson, s_srs)?;
        db.delete_collection(&collection.id).await?;
        db.create_collection(&collection).await?;

        // Load features
        let count = geojson.features.len();
        let chunk_size = 8192;

        let mut ids = Vec::with_capacity(chunk_size);
        let mut properties = Vec::with_capacity(chunk_size);
        let mut geoms = Vec::with_capacity(chunk_size);

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
            geoms.push(feature.geometry.as_ref().unwrap().value.to_string());

            // write chunk
            if ids.len() == chunk_size {
                bulk_insert(&collection.id, &ids, &properties, &geoms, &db.pool).await?;

                ids.clear();
                properties.clear();
                geoms.clear();
            }
        }

        if !ids.is_empty() {
            bulk_insert(&collection.id, &ids, &properties, &geoms, &db.pool).await?;
        }

        // update extent
        if collection.extent.is_none() {
            tracing::info!("Set spatial extent");
            sqlx::query(&format!(r#"
                UPDATE meta.collections 
                SET collection = (
                    collection || (
                        SELECT
                            jsonb_build_object('extent',
                            jsonb_build_object('spatial',
                            jsonb_build_object(
                                'bbox',
                                (
                                    WITH extent AS (
                                        SELECT ST_Extent(geom) AS e FROM items."{0}"
                                    )
                                    SELECT ARRAY[ARRAY[ST_XMin(e), ST_YMin(e), ST_XMax(e), ST_YMax(e)]]
                                    FROM extent
                                ),
                                'crs',
                                collection -> 'storageCrs'
                            )))
                        )
                    ) 
                WHERE id = '{0}'
                "#,
                collection.id)
            )
            .execute(&db.pool)
            .await?;
        }

        // stats
        let elapsed = now.elapsed().as_millis() as f64 / 1000.0;
        tracing::info!(
            "Loaded {count} features in {elapsed} seconds ({:.2}/s)",
            count as f64 / elapsed
        );

        Ok(())
    }

    async fn bulk_insert(
        collection_id: &str,
        ids: &[String],
        properties: &[Option<Json<Map<String, Value>>>],
        geoms: &[String],
        pool: &Pool<Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query(&format!(
            r#"
        INSERT INTO items."{}" (id, properties, geom)
        SELECT id, properties, ST_SetSRID(ST_GeomFromGeoJSON(geom), (SELECT Find_SRID('items', '{0}', 'geom')))
        FROM UNNEST($1::text[], $2::jsonb[], $3::text[]) AS t(id, properties, geom)
        "#,
            collection_id
        ))
        .bind(ids)
        .bind(properties)
        .bind(geoms)
        .execute(pool)
        .await
    }
}

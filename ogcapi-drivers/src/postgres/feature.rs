use anyhow::Context;
use ogcapi_types::{
    common::{Bbox, Crs, Exception},
    features::{Feature, FeatureCollection, Query},
};

use crate::{CollectionTransactions, FeatureTransactions, not_found};

use super::{Db, datetime_condition, envelopes, items_table};

/// The columns of a feature, in the system of `$1`.
macro_rules! rows {
    () => {
        "items.id, items.collection, properties, \
        ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb AS geometry, links"
    };
}

#[cfg(not(feature = "stac"))]
const ROWS: &str = rows!();

/// With `stac`, those of an item too: its box, where it has a geometry.
#[cfg(feature = "stac")]
const ROWS: &str = concat!(
    rows!(),
    ", meta.collection ->> 'stac_version' AS stac_version, \
    COALESCE(meta.collection -> 'stac_extensions', '[]'::jsonb) AS stac_extensions, \
    assets, \
    COALESCE(bbox, CASE WHEN geom IS NOT NULL THEN array_to_json(ARRAY[ \
        st_xmin(st_transform(geom, 4326)::box2d), \
        st_ymin(st_transform(geom, 4326)::box2d), \
        st_xmax(st_transform(geom, 4326)::box2d), \
        st_ymax(st_transform(geom, 4326)::box2d) \
    ])::jsonb END) AS bbox"
);

/// A feature's geometry from its `GeoJSON` in `$1`, CRS84 (Part 4,
/// /req/features/default-crs), in the system of the collection `$2`.
const GEOMETRY: &str = "ST_Transform(ST_SetSRID(ST_GeomFromGeoJSON(NULLIF($1 -> 'geometry', 'null')), 4326), (SELECT Find_SRID('items', $2, 'geom')))";

#[allow(clippy::too_many_lines, reason = "Complex transaction logic")]
#[async_trait::async_trait]
impl FeatureTransactions for Db {
    async fn create_feature(&self, feature: &Feature) -> anyhow::Result<String> {
        let collection_id = feature
            .collection
            .as_ref()
            .context("feature.collection is required")?;
        let table = items_table(collection_id).ok_or_else(not_found)?;

        let id: (String,) = sqlx::query_as(&format!(
            r"
            INSERT INTO {table} (
                id,
                properties,
                geom,
                links,
                assets,
                bbox
            ) VALUES (
                COALESCE($1 ->> 'id', gen_random_uuid()::text),
                $1 -> 'properties',
                {GEOMETRY},
                $1 -> 'links',
                COALESCE($1 -> 'assets', '{{}}'::jsonb),
                $1 -> 'bbox'
            )
            RETURNING id
            ",
        ))
        .bind(serde_json::to_value(feature)?)
        .bind(collection_id)
        .fetch_one(&self.pool)
        .await
        .map_err(conflict)?;

        Ok(id.0)
    }

    async fn read_feature(
        &self,
        collection: &str,
        id: &str,
        crs: &Crs,
    ) -> anyhow::Result<Option<Feature>> {
        let Some(table) = items_table(collection) else {
            return Ok(None);
        };

        let feature: Option<sqlx::types::Json<Feature>> = sqlx::query_scalar(&format!(
            r"
            SELECT row_to_json(t)
            FROM (
                SELECT {ROWS}
                FROM {table} items JOIN meta.collections meta
                    ON items.collection = meta.id
                WHERE items.id = $2
            ) t
            "
        ))
        .bind(crs.as_srid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(feature.map(|f| f.0))
    }

    async fn update_feature(&self, feature: &Feature) -> anyhow::Result<()> {
        let collection_id = feature
            .collection
            .as_ref()
            .context("feature.collection is required")?;
        let table = items_table(collection_id).ok_or_else(not_found)?;

        let updated = sqlx::query(&format!(
            r"
            UPDATE {table}
            SET
                properties = $1 -> 'properties',
                geom = {GEOMETRY},
                links = $1 -> 'links',
                assets = COALESCE($1 -> 'assets', '{{}}'::jsonb)
            WHERE id = $1 ->> 'id'
            "
        ))
        .bind(serde_json::to_value(feature)?)
        .bind(collection_id)
        .execute(&self.pool)
        .await?;

        if updated.rows_affected() == 0 {
            return Err(not_found());
        }

        Ok(())
    }

    async fn delete_feature(&self, collection: &str, id: &str) -> anyhow::Result<()> {
        let table = items_table(collection).ok_or_else(not_found)?;

        let deleted = sqlx::query(&format!("DELETE FROM {table} WHERE id = $1"))
            .bind(id)
            .execute(&self.pool)
            .await?;

        if deleted.rows_affected() == 0 {
            return Err(not_found());
        }

        Ok(())
    }

    async fn list_items(
        &self,
        collection_id: &str,
        query: &Query,
    ) -> anyhow::Result<FeatureCollection> {
        let table = items_table(collection_id).ok_or_else(not_found)?;

        let mut where_conditions = vec!["TRUE".to_owned()];

        // bbox
        if let Some(bbox) = query.bbox.as_ref() {
            // crs; coordinates in the order PostGIS takes them, the longitude
            // first, whatever the order of the system's definition
            let bbox_crs = query.bbox_crs.clone().unwrap_or_else(|| match bbox {
                Bbox::Bbox2D(_) => Crs::default2d(),
                Bbox::Bbox3D(_) => Crs::default3d(),
            });
            let srid = bbox_crs.as_srid().context("cannot determine SRID")?;

            let collection = self
                .read_collection(collection_id)
                .await?
                .ok_or_else(not_found)?;
            let storage_srid = self.storage_srid(&collection).await?;

            let intersection = match bbox {
                Bbox::Bbox2D(_) => {
                    let intersects: Vec<String> = envelopes(bbox, srid)
                        .iter()
                        .map(|envelope| {
                            format!("ST_Intersects(geom, ST_Transform({envelope}, {storage_srid}))")
                        })
                        .collect();
                    format!("(geom IS NULL OR {})", intersects.join(" OR "))
                }
                Bbox::Bbox3D(bbox) => format!(
                    // FIXME: ensure proper height/box transformation handling
                    r"(geom IS NULL OR ST_3DIntersects(geom, ST_Envelope(ST_Transform(ST_SetSRID(ST_MakeLine(ARRAY[
                        ST_MakePoint({x1}, {y1}, {z1}), 
                        ST_MakePoint({x2}, {y1}, {z1}),
                        ST_MakePoint({x1}, {y2}, {z1}), 
                        ST_MakePoint({x2}, {y2}, {z1}),
                        ST_MakePoint({x1}, {y1}, {z2}), 
                        ST_MakePoint({x2}, {y1}, {z2}),
                        ST_MakePoint({x1}, {y2}, {z2}), 
                        ST_MakePoint({x2}, {y2}, {z2})
                    ]), {srid}), {storage_srid}))))",
                    x1 = bbox[0],
                    y1 = bbox[1],
                    z1 = bbox[2],
                    x2 = bbox[3],
                    y2 = bbox[4],
                    z2 = bbox[5],
                ),
            };

            where_conditions.push(intersection);
        }

        // datetime
        if let Some(datetime) = query.datetime.as_ref() {
            where_conditions.push(datetime_condition(datetime));
        }

        // kv, each key and value bound, from placeholder `first` on
        let kv: Vec<(&String, &String)> = query.additional_parameters.iter().collect();
        let conditions = |first: usize| {
            let mut conditions = where_conditions.clone();
            for i in 0..kv.len() {
                let (k, v) = (first + 2 * i, first + 2 * i + 1);
                conditions.push(format!(
                    "CASE WHEN properties ? ${k} THEN properties ->> ${k} = ${v} ELSE TRUE END"
                ));
            }
            conditions.join(" AND ")
        };

        // count
        let sql = format!("SELECT count(*) FROM {table} WHERE {}", conditions(1));
        let mut count = sqlx::query_as(&sql);
        for (k, v) in &kv {
            count = count.bind(k.as_str()).bind(v.as_str());
        }
        let number_matched: (i64,) = count.fetch_one(&self.pool).await?;

        // srid
        let srid = query
            .crs
            .as_ref()
            .map_or_else(|| Crs::default2d().as_srid(), Crs::as_srid);

        // fetch
        let sql = format!(
            r"
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM (
                SELECT {ROWS}
                FROM {table} items JOIN meta.collections meta
                    ON items.collection = meta.id
                WHERE {}
                ORDER BY items.id
                LIMIT {}
                OFFSET {}
            ) t
            ",
            conditions(2),
            query
                .limit
                .map_or_else(|| String::from("NULL"), |l| l.to_string()),
            query.offset.unwrap_or(0)
        );
        let mut fetch = sqlx::query_scalar(&sql).bind(srid);
        for (k, v) in &kv {
            fetch = fetch.bind(k.as_str()).bind(v.as_str());
        }
        let features: Option<sqlx::types::Json<Vec<Feature>>> = fetch.fetch_one(&self.pool).await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched.0.cast_unsigned());

        Ok(fc)
    }
}

/// A write's error, a refusal (409) of a feature whose id another has.
fn conflict(error: sqlx::Error) -> anyhow::Error {
    if error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
    {
        Exception::new_from_status(409)
            .detail("a feature of this id exists")
            .into()
    } else {
        error.into()
    }
}

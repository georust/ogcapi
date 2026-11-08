use anyhow::Context;
use ogcapi_types::{
    common::{Authority, Bbox, Crs, Datetime, Exception, IntervalDatetime},
    features::{Feature, FeatureCollection, Query},
};

use crate::{CollectionTransactions, FeatureTransactions};

use super::{Db, items_table};

#[cfg(not(feature = "stac"))]
static ROWS: &str = "
items.id,
items.collection,
properties,
ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb AS geometry,
links
";

#[cfg(feature = "stac")]
static ROWS: &str = "
items.id,
items.collection,
properties,
ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb AS geometry,
links,
meta.collection ->> 'stac_version' AS stac_version,
COALESCE(
    (meta.collection -> 'stac_extensions'),
    '[]'::jsonb
) AS stac_extensions,
assets,
COALESCE(
    bbox,
    array_to_json(
        ARRAY[
            st_xmin(st_transform(geom, 4326)::box2d),
            st_ymin(st_transform(geom, 4326)::box2d),
            st_xmax(st_transform(geom, 4326)::box2d),
            st_ymax(st_transform(geom, 4326)::box2d)
        ]
    )::jsonb
) as bbox
";

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
                ST_Transform(ST_SetSRID(ST_GeomFromGeoJSON(NULLIF($1 -> 'geometry', 'null')), 4326), (SELECT Find_SRID('items', $2, 'geom'))),
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
        .await?;

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
                geom = ST_Transform(ST_SetSRID(ST_GeomFromGeoJSON(NULLIF($1 -> 'geometry', 'null')), 4326), (SELECT Find_SRID('items', $2, 'geom'))),
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
            // crs
            let bbox_crs = query.bbox_crs.clone().unwrap_or_else(|| match bbox {
                Bbox::Bbox2D(_) => Crs::default2d(),
                Bbox::Bbox3D(_) => Crs::default3d(),
            });

            // coordinate system axis order (OGC and Postgis is lng, lat | EPSG is lat, lng)
            let order = match bbox_crs.authority {
                Authority::OGC | Authority::EPSG => [0, 1, 2, 3], // Use "traditional GIS order". TODO: should follow the crs definition [1, 0, 3, 2]
            };

            let collection = self.read_collection(collection_id).await?;
            let storage_srid = match collection {
                Some(collection) => match collection.storage_crs.map(|crs| crs.as_srid()) {
                    Some(srid) => srid.context("cannot determine SRID")?,
                    None => {
                        sqlx::query_scalar("SELECT Find_SRID('items', $1, 'geom')")
                            .bind(collection_id)
                            .fetch_one(&self.pool)
                            .await?
                    }
                },
                None => return Err(not_found()),
            };

            // TODO: handle antimeridian (lower > upper on axis 1)
            let intersection = match bbox {
                Bbox::Bbox2D(bbox) => format!(
                    "(geom IS NULL OR ST_Intersects(geom, ST_Transform(ST_MakeEnvelope({}, {}, {}, {}, {}), {storage_srid})))",
                    bbox[order[0]],
                    bbox[order[1]],
                    bbox[order[2]],
                    bbox[order[3]],
                    bbox_crs.as_srid().context("cannot determine SRID")?
                ),
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
                    x1 = bbox[order[0]],
                    y1 = bbox[order[1]],
                    z1 = bbox[2],
                    x2 = bbox[order[2] + 1],
                    y2 = bbox[order[3] + 1],
                    z2 = bbox[5],
                    srid = bbox_crs.as_srid().context("cannot determine SRID")?
                ),
            };

            where_conditions.push(intersection);
        }

        // datetime
        if let Some(datetime) = query.datetime.as_ref() {
            let (from, to) = match datetime {
                Datetime::Datetime(_) => (
                    format!("CAST('{datetime}' AS timestamptz)"),
                    format!("CAST('{datetime}' AS timestamptz)"),
                ),
                Datetime::Interval { from, to } => {
                    let from = match from {
                        IntervalDatetime::Datetime(_) => {
                            format!("CAST('{from}' AS timestamptz)")
                        }
                        IntervalDatetime::Open => "to_timestamp('-infinity')".to_owned(),
                    };
                    let to = match to {
                        IntervalDatetime::Datetime(_) => {
                            format!("CAST('{to}' AS timestamptz)")
                        }
                        IntervalDatetime::Open => "NOW()".to_owned(),
                    };
                    (from, to)
                }
            };

            where_conditions.push(format!(
                r"
                (
                    CASE
                        WHEN (properties->'datetime') IS NOT NULL THEN (
                            CAST(properties->>'datetime' AS timestamptz)
                            BETWEEN {from} AND {to}
                        )
                        WHEN (
                            (properties->'datetime') IS NULL
                            AND (properties->'start_datetime') IS NOT NULL
                            AND (properties->'end_datetime') IS NOT NULL
                        ) THEN (
                            ({from}, {to}) OVERLAPS (
                                CAST(properties->>'start_datetime' AS timestamptz),
                                CAST(properties->>'end_datetime' AS timestamptz)
                            )
                        )
                        ELSE TRUE
                    END
                )
                "
            ));
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

/// A collection or feature that doesn't exist.
fn not_found() -> anyhow::Error {
    Exception::new_from_status(404).into()
}

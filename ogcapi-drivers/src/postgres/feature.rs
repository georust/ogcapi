use ogcapi_types::{
    common::{Authority, Bbox, Crs, Datetime, Exception, IntervalDatetime},
    features::{Feature, FeatureCollection, Query},
};

use crate::{CollectionTransactions, FeatureTransactions};

use super::Db;

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

#[async_trait::async_trait]
impl FeatureTransactions for Db {
    async fn create_feature(&self, feature: &Feature) -> anyhow::Result<String> {
        let collection = feature.collection.as_ref().unwrap();

        let id: (String,) = sqlx::query_as(&format!(
            r#"
            INSERT INTO items."{0}" (
                id,
                properties,
                geom,
                links,
                assets,
                bbox
            ) VALUES (
                COALESCE($1 ->> 'id', gen_random_uuid()::text),
                $1 -> 'properties',
                ST_GeomFromGeoJSON($1 -> 'geometry'),
                $1 -> 'links',
                COALESCE($1 -> 'assets', '{{}}'::jsonb),
                $1 -> 'bbox'
            )
            RETURNING id
            "#,
            &collection
        ))
        .bind(serde_json::to_value(feature)?)
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
        let feature: Option<sqlx::types::Json<Feature>> = sqlx::query_scalar(&format!(
            r#"
            SELECT row_to_json(t)
            FROM (
                SELECT {ROWS}
                FROM items."{collection}" items JOIN meta.collections meta
                    ON items.collection = meta.id
                WHERE items.id = $2
            ) t
            "#
        ))
        .bind(crs.as_srid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(feature.map(|f| f.0))
    }

    async fn update_feature(&self, feature: &Feature) -> anyhow::Result<()> {
        sqlx::query(&format!(
            r#"
            UPDATE items."{0}"
            SET
                properties = $1 -> 'properties',
                geom = ST_GeomFromGeoJSON($1 -> 'geometry'),
                links = $1 -> 'links',
                assets = COALESCE($1 -> 'assets', '{{}}'::jsonb)
            WHERE id = $1 ->> 'id'
            "#,
            &feature.collection.as_ref().unwrap()
        ))
        .bind(serde_json::to_value(feature)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_feature(&self, collection: &str, id: &str) -> anyhow::Result<()> {
        sqlx::query(&format!(
            r#"DELETE FROM items."{collection}" WHERE id = $1"#
        ))
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_items(
        &self,
        collection_id: &str,
        query: &Query,
    ) -> anyhow::Result<FeatureCollection> {
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
                Authority::OGC => [0, 1, 2, 3],
                Authority::EPSG => [1, 0, 3, 2],
            };

            let collection = self.read_collection(collection_id).await?;
            let storage_srid = match collection {
                Some(collection) => match collection.storage_crs.map(|crs| crs.as_srid()) {
                    Some(srid) => srid,
                    None => {
                        sqlx::query_scalar(&format!(
                            "SELECT Find_SRID('items', '{collection_id}', 'geom')"
                        ))
                        .fetch_one(&self.pool)
                        .await?
                    }
                },
                None => return Err(Exception::new_from_status(404).into()),
            };

            // TODO: handle antimeridian (lower > upper on axis 1)
            let intersection = match bbox {
                Bbox::Bbox2D(bbox) => format!(
                    "ST_Intersects(geom, ST_Transform(ST_MakeEnvelope({}, {}, {}, {}, {}), {storage_srid}))",
                    bbox[order[0]],
                    bbox[order[1]],
                    bbox[order[2]],
                    bbox[order[3]],
                    bbox_crs.as_srid()
                ),
                Bbox::Bbox3D(bbox) => format!(
                    // FIXME: ensure proper height/box transformation handling
                    r#"ST_3DIntersects(geom, ST_Envelope(ST_Transform(ST_SetSRID(ST_MakeLine(ARRAY[
                        ST_MakePoint({x1}, {y1}, {z1}), 
                        ST_MakePoint({x2}, {y1}, {z1}),
                        ST_MakePoint({x1}, {y2}, {z1}), 
                        ST_MakePoint({x2}, {y2}, {z1}),
                        ST_MakePoint({x1}, {y1}, {z2}), 
                        ST_MakePoint({x2}, {y1}, {z2}),
                        ST_MakePoint({x1}, {y2}, {z2}), 
                        ST_MakePoint({x2}, {y2}, {z2})
                    ]), {srid}), {storage_srid})))"#,
                    x1 = bbox[order[0]],
                    y1 = bbox[order[1]],
                    z1 = bbox[2],
                    x2 = bbox[order[2] + 1],
                    y2 = bbox[order[3] + 1],
                    z2 = bbox[5],
                    srid = bbox_crs.as_srid()
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
                r#"
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
                "#
            ));
        }

        // kv
        for (k, v) in query.additional_parameters.iter() {
            where_conditions.push(format!(
                r#"
                CASE
                    WHEN properties ? '{k}' THEN (
                        CASE
                            WHEN jsonb_typeof(properties -> '{k}') = 'number'
                            THEN RTRIM(properties ->> '{k}', '.0') = RTRIM('{v}', '.0')
                            ELSE properties ->> '{k}' = '{v}'
                        END
                    ) 
                    ELSE TRUE
                END
                "#
            ));
        }

        let conditions = where_conditions.join(" AND ");

        // count
        let number_matched: (i64,) = sqlx::query_as(&format!(
            r#"
            SELECT count(*) FROM items."{collection_id}"
            WHERE {conditions}
            "#,
        ))
        .fetch_one(&self.pool)
        .await?;

        // srid
        let srid = query
            .crs
            .as_ref()
            .map(|crs| crs.as_srid())
            .unwrap_or_else(|| Crs::default2d().as_srid());

        // fetch
        let features: Option<sqlx::types::Json<Vec<Feature>>> = sqlx::query_scalar(&format!(
            r#"
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM (
                SELECT {ROWS}
                FROM items."{collection_id}" items JOIN meta.collections meta
                    ON items.collection = meta.id
                WHERE {conditions}
                LIMIT {}
                OFFSET {}
            ) t
            "#,
            query
                .limit
                .map_or_else(|| String::from("NULL"), |l| l.to_string()),
            query.offset.unwrap_or(0)
        ))
        .bind(srid)
        .fetch_one(&self.pool)
        .await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched.0 as u64);

        Ok(fc)
    }
}

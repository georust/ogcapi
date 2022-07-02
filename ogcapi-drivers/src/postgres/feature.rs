use ogcapi_types::{
    common::{Bbox, Crs},
    features::{Feature, FeatureCollection, Query},
};

use crate::{CollectionTransactions, FeatureTransactions};

use super::Db;

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
        .bind(serde_json::to_value(&feature)?)
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
                SELECT
                    id,
                    '{0}' AS collection,
                    properties,
                    ST_AsGeoJSON(ST_Transform(geom, $2::int))::jsonb as geometry,
                    links,
                    assets,
                    bbox
                FROM items."{0}"
                WHERE id = $1
            ) t
            "#,
            collection
        ))
        .bind(id)
        .bind(crs.as_srid())
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
                assets = COALESCE($1 -> 'assets', '{{}}'::jsonb),
                bbox = $1 -> 'bbox'
            WHERE id = $1 ->> 'id'
            "#,
            &feature.collection.as_ref().unwrap()
        ))
        .bind(serde_json::to_value(&feature)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_feature(&self, collection: &str, id: &str) -> anyhow::Result<()> {
        sqlx::query(&format!(
            r#"DELETE FROM items."{}" WHERE id = $1"#,
            collection
        ))
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_items(
        &self,
        collection: &str,
        query: &Query,
    ) -> anyhow::Result<FeatureCollection> {
        let mut where_conditions = vec!["TRUE".to_owned()];

        // bbox
        if let Some(bbox) = query.bbox.as_ref() {
            // TODO: Properly handle crs and bbox transformation
            let bbox_srid: i32 = query.bbox_crs.as_srid();

            let c = self.read_collection(collection).await?;
            let storage_srid = c
                .expect("collection exists")
                .storage_crs
                .unwrap_or_default()
                .as_srid();

            let envelope = match bbox {
                Bbox::Bbox2D(bbox) => format!(
                    "ST_MakeEnvelope({}, {}, {}, {}, {})",
                    bbox[0], bbox[1], bbox[2], bbox[3], bbox_srid
                ),
                Bbox::Bbox3D(bbox) => format!(
                    "ST_MakeEnvelope({}, {}, {}, {}, {})",
                    bbox[0], bbox[1], bbox[3], bbox[4], bbox_srid
                ),
            };
            where_conditions.push(format!(
                "geom && ST_Transform({}, {})",
                envelope, storage_srid
            ));
        }

        let conditions = where_conditions.join(" AND ");

        // count
        let number_matched: (i64,) = sqlx::query_as(&format!(
            r#"
            SELECT count(*) FROM items."{collection}"
            WHERE {conditions}
            "#,
        ))
        .fetch_one(&self.pool)
        .await?;

        // fetch
        let features: Option<sqlx::types::Json<Vec<Feature>>> = sqlx::query_scalar(&format!(
            r#"
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM (
                SELECT
                    id,
                    '{collection}' as collection,
                    properties,
                    ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
                    links,
                    assets,
                    bbox
                FROM items."{collection}"
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
        .bind(query.crs.as_srid())
        .fetch_one(&self.pool)
        .await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched.0 as u64);

        Ok(fc)
    }
}

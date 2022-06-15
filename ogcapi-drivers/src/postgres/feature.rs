use async_trait::async_trait;
use sqlx::types::Json;

use ogcapi_types::{
    common::{Bbox, Crs},
    features::{Feature, FeatureCollection, Query},
};

use crate::{CollectionTransactions, FeatureTransactions};

use super::Db;

#[async_trait]
impl FeatureTransactions for Db {
    async fn create_feature(&self, feature: &Feature) -> Result<String, anyhow::Error> {
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
    ) -> Result<Feature, anyhow::Error> {
        let feature: Json<Feature> = sqlx::query_scalar(&format!(
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
        .fetch_one(&self.pool)
        .await?;

        Ok(feature.0)
    }

    async fn update_feature(&self, feature: &Feature) -> Result<(), anyhow::Error> {
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

    async fn delete_feature(&self, collection: &str, id: &str) -> Result<(), anyhow::Error> {
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
    ) -> Result<FeatureCollection, anyhow::Error> {
        let mut sql = vec![format!(
            r#"
            SELECT
                id,
                '{0}' as collection,
                properties,
                ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
                links,
                assets,
                bbox
            FROM items."{0}"
            "#,
            collection
        )];

        if let Some(bbox) = query.bbox.as_ref() {
            // TODO: Properly handle crs and bbox transformation
            let bbox_srid: i32 = query.bbox_crs.as_srid();

            let c = self.read_collection(collection).await?;
            let storage_srid = c.storage_crs.unwrap_or_default().as_srid();

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
            sql.push(format!(
                "WHERE geom && ST_Transform({}, {})",
                envelope, storage_srid
            ));
        }

        let srid = query.crs.as_srid();

        let number_matched = sqlx::query(sql.join(" ").as_str())
            .bind(srid)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if let Some(limit) = query.limit {
            sql.push(format!("LIMIT {}", limit));
            if let Some(offset) = query.offset {
                sql.push(format!("OFFSET {}", offset));
            }
        }

        let features: Option<Json<Vec<Feature>>> = sqlx::query_scalar(&format!(
            r#"
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM ( {} ) t
            "#,
            sql.join(" ")
        ))
        .bind(srid)
        .fetch_one(&self.pool)
        .await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched);

        Ok(fc)
    }
}

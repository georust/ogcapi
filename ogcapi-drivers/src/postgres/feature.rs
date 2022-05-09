use anyhow::Ok;
use async_trait::async_trait;
use sqlx::types::Json;

use ogcapi_types::{
    common::{Bbox, Crs},
    features::{Feature, FeatureCollection, Query},
};

use crate::FeatureTransactions;

use super::Db;

#[async_trait]
impl FeatureTransactions for Db {
    async fn create_feature(&self, feature: &Feature) -> Result<String, anyhow::Error> {
        let collection = feature.collection.as_ref().unwrap();

        let id: (String,) = sqlx::query_as(&format!(
            r#"
            INSERT INTO items.{0} (
                type,
                properties,
                geom,
                links
            ) VALUES ($1, $2, ST_GeomFromGeoJSON($3), $4)
            RETURNING id
            "#,
            &collection
        ))
        .bind(&feature.r#type)
        .bind(serde_json::to_value(&feature.properties)?)
        .bind(serde_json::to_value(&feature.geometry)?)
        .bind(serde_json::to_value(&feature.links)?)
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("collections/{}/items/{}", &collection, id.0))
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
                    type,
                    properties,
                    ST_AsGeoJSON(ST_Transform(geom, $2::int))::jsonb as geometry,
                    links
                FROM items.{0}
                WHERE id = $1
            ) t
            "#,
            collection
        ))
        .bind(id)
        .bind(crs.to_owned().try_into().unwrap_or(4326))
        .fetch_one(&self.pool)
        .await?;

        Ok(feature.0)
    }

    async fn update_feature(&self, feature: &Feature) -> Result<(), anyhow::Error> {
        sqlx::query(&format!(
            r#"
            UPDATE items.{0}
            SET
                type = $2,
                properties = $3,
                geom = ST_GeomFromGeoJSON($4),
                links = $5
            WHERE id = $1
            "#,
            &feature.collection.as_ref().unwrap()
        ))
        .bind(&feature.id)
        .bind(&feature.r#type)
        .bind(serde_json::to_value(&feature.properties)?)
        .bind(serde_json::to_value(&feature.geometry)?)
        .bind(serde_json::to_value(&feature.links)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_feature(&self, collection: &str, id: &str) -> Result<(), anyhow::Error> {
        sqlx::query(&format!("DELETE FROM items.{} WHERE id = $1", collection))
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
                type,
                properties,
                ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
                links,
                '{0}' as collection
            FROM items.{0}
            "#,
            collection
        )];

        if let Some(bbox) = query.bbox.as_ref() {
            let bbox_srid: i32 = query
                .bbox_crs
                .to_owned()
                .unwrap_or_default()
                .try_into()
                .unwrap();

            let storage_srid = self.storage_srid(collection).await?;

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

        let srid = query
            .crs
            .to_owned()
            .unwrap_or_default()
            .try_into()
            .unwrap_or(4326);

        let number_matched = sqlx::query(sql.join(" ").as_str())
            .bind(srid)
            .execute(&self.pool)
            .await?
            .rows_affected();

        let features: Option<Json<Vec<Feature>>> = sqlx::query_scalar(&format!(
            r#"
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM ( {} ) t
            "#,
            sql.join(" ")
        ))
        .bind(&srid)
        .fetch_one(&self.pool)
        .await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched);

        Ok(fc)
    }
}

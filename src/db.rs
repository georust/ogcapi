use sqlx::postgres::PgRow;
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, types::Json, Pool, Postgres};

use crate::common::collections::Collection;
use crate::common::core::{Conformance, Link, Links};
use crate::features::{Assets, Feature, FeatureType, Geometry};

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Pool<Postgres>,
}

impl Db {
    pub async fn connect(url: &str) -> Result<Self, anyhow::Error> {
        let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;

        Ok(Db { pool })
    }

    pub async fn root(&self) -> Result<Links, anyhow::Error> {
        let links = sqlx::query("SELECT row_to_json(root) FROM meta.root")
            .try_map(|row: PgRow| {
                serde_json::from_value::<Link>(row.get(0))
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(links)
    }

    pub async fn conformance(&self) -> Result<Conformance, anyhow::Error> {
        let classes = sqlx::query_scalar!("SELECT * FROM meta.conformance")
            .fetch_all(&self.pool)
            .await?;

        Ok(Conformance {
            conforms_to: classes,
        })
    }

    pub async fn insert_collection(
        &self,
        collection: &Collection,
    ) -> Result<String, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS items.{0} (
                id bigserial PRIMARY KEY,
                feature_type jsonb NOT NULL DEFAULT '"Feature"'::jsonb,
                properties jsonb,
                geom geometry NOT NULL,
                links jsonb,
                stac_version text,
                stac_extensions text[],
                assets jsonb
            )
            "#,
            collection.id
        ))
        .execute(&mut tx)
        .await?;

        sqlx::query(&format!(
            "CREATE INDEX ON items.{0} USING gin (properties)",
            collection.id
        ))
        .execute(&mut tx)
        .await?;

        sqlx::query(&format!(
            "CREATE INDEX ON items.{0} USING gist (geom)",
            collection.id
        ))
        .execute(&mut tx)
        .await?;

        sqlx::query(&format!(
            "SELECT UpdateGeometrySRID('items', '{0}', 'geom', 4326)",
            collection.id
        ))
        .execute(&mut tx)
        .await?;

        sqlx::query("INSERT INTO meta.collections ( id, collection ) VALUES ( $1, $2 )")
            .bind(&collection.id)
            .bind(Json(collection) as Json<&Collection>)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(format!("collections/{}", collection.id))
    }

    pub async fn select_collection(&self, id: &str) -> Result<Collection, anyhow::Error> {
        let collection: (Json<Collection>,) =
            sqlx::query_as("SELECT collection FROM meta.collections WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(collection.0 .0)
    }

    pub async fn update_collection(&self, collection: &Collection) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE meta.collections SET collection = $2 WHERE id = $1")
            .bind(&collection.id)
            .bind(Json(collection) as Json<&Collection>)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_collection(&self, id: &str) -> Result<(), anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(&format!("DROP TABLE IF EXISTS items.{}", id))
            .execute(&mut tx)
            .await?;

        sqlx::query("DELETE FROM meta.collections WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn insert_feature(&self, feature: &Feature) -> Result<String, anyhow::Error> {
        let collection = feature.collection.as_ref().unwrap();

        let id: (i64,) = sqlx::query_as(&format!(
            r#"
            INSERT INTO items.{0} (
                feature_type,
                properties,
                geom,
                links,
                stac_version,
                stac_extensions,
                assets
            ) VALUES ($1, $2, ST_SetSRID(ST_GeomFromGeoJSON($3),4326), $4, $5, $6, $7)
            RETURNING id
            "#,
            &collection
        ))
        .bind(&feature.feature_type as &Json<FeatureType>)
        .bind(&feature.properties)
        .bind(&feature.geometry as &Json<Geometry>)
        .bind(&feature.links as &Option<Json<Links>>)
        .bind(&feature.stac_version)
        .bind(&feature.stac_extensions.as_deref())
        .bind(&feature.assets as &Option<Json<Assets>>)
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("collections/{}/items/{}", &collection, id.0))
    }

    pub async fn select_feature(
        &self,
        collection: &str,
        id: &i64,
        crs: Option<i32>,
    ) -> Result<Feature, anyhow::Error> {
        // '{0}' AS "collection?",
        let feature: Feature = sqlx::query_as(&format!(
            r#"
            SELECT
                id,
                '{0}' AS collection,
                feature_type,
                properties,
                ST_AsGeoJSON(ST_Transform(geom, $2::int))::jsonb as geometry,
                links,
                stac_version,
                stac_extensions,
                ST_AsGeoJSON(ST_Transform(geom, $2::int), 9, 1)::jsonb -> 'bbox' AS bbox,
                assets
            FROM items.{0}
            WHERE id = $1
            "#,
            collection
        ))
        .bind(id)
        .bind(crs.unwrap_or(4326))
        .fetch_one(&self.pool)
        .await?;

        Ok(feature)
    }

    pub async fn update_feature(&self, feature: &Feature) -> Result<(), anyhow::Error> {
        sqlx::query(&format!(
            r#"
            UPDATE items.{0}
            SET
                feature_type = $2,
                properties = $3,
                geom = ST_GeomFromGeoJSON($4),
                links = $5,
                stac_version = $6,
                stac_extensions = $7,
                assets = $8
            WHERE id = $1
            "#,
            &feature.collection.as_ref().unwrap()
        ))
        .bind(&feature.id)
        .bind(&feature.feature_type as &Json<FeatureType>)
        .bind(&feature.properties)
        .bind(&feature.geometry as &Json<Geometry>)
        .bind(&feature.links as &Option<Json<Links>>)
        .bind(&feature.stac_version)
        .bind(&feature.stac_extensions.as_deref())
        .bind(&feature.assets as &Option<Json<Assets>>)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_feature(&self, collection: &str, id: &i64) -> Result<(), anyhow::Error> {
        sqlx::query(&format!("DELETE FROM items.{} WHERE id = $1", collection))
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use serde_json::json;

    use crate::common::collections::Collection;
    use crate::common::core::Link;
    use crate::db::Db;
    use crate::features::Feature;

    #[async_std::test]
    async fn minimal_feature_crud() -> Result<(), anyhow::Error> {
        // setup env
        dotenv::dotenv().ok();

        let db = Db::connect(&env::var("DATABASE_URL")?).await?;

        let collection = Collection {
            id: "test".to_string(),
            links: vec![Link {
                href: "collections/test".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        // create collection
        let location = db.insert_collection(&collection).await?;
        println!("{}", location);

        let feature: Feature = serde_json::from_value(json!({
            "collection": "test",
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [7.428959, 1.513394]
            },
            "links": [{
                "href": "collections/test/items/{id}",
                "rel": "self"
            }]
        }))
        .unwrap();

        // create feature
        let location = db.insert_feature(&feature).await?;
        println!("{}", location);

        let id = location.split("/").last().unwrap().parse()?;

        // read feauture
        let feature = db.select_feature(&collection.id, &id, None).await?;
        // println!("{:#?}", feature);

        // update
        db.update_feature(&feature).await?;

        // delete feature
        db.delete_feature(&collection.id, &id).await?;

        // delete collection
        db.delete_collection(&collection.id).await?;

        Ok(())
    }
}

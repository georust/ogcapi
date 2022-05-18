use async_trait::async_trait;
use ogcapi_types::common::{Collection, Collections, Query};
use sqlx::types::Json;

use crate::CollectionTransactions;

use super::Db;

#[async_trait]
impl CollectionTransactions for Db {
    async fn create_collection(&self, collection: &Collection) -> Result<String, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS items.{0} (
                id text PRIMARY KEY DEFAULT gen_random_uuid()::text,
                type text NOT NULL DEFAULT 'Feature',
                properties jsonb,
                geom geometry NOT NULL,
                links jsonb NOT NULL DEFAULT '[]'::jsonb
            )
            "#,
            collection.id
        ))
        .execute(&mut tx)
        .await?;

        sqlx::query(&format!(
            "CREATE INDEX ON items.{} USING gin (properties)",
            collection.id
        ))
        .execute(&mut tx)
        .await?;

        sqlx::query(&format!(
            "CREATE INDEX ON items.{} USING gist (geom)",
            collection.id
        ))
        .execute(&mut tx)
        .await?;

        sqlx::query("SELECT UpdateGeometrySRID('items', $1, 'geom', $2)")
            .bind(&collection.id)
            .bind(
                &collection
                    .storage_crs
                    .clone()
                    .and_then(|c| c.try_into().ok())
                    .unwrap_or(4326),
            )
            .execute(&mut tx)
            .await?;

        sqlx::query("INSERT INTO meta.collections ( id, collection ) VALUES ( $1, $2 )")
            .bind(&collection.id)
            .bind(Json(collection))
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(collection.id.to_owned())
    }

    async fn read_collection(&self, id: &str) -> Result<Collection, anyhow::Error> {
        let collection = sqlx::query_scalar!(
            r#"
            SELECT collection as "collection!: sqlx::types::Json<Collection>" 
            FROM meta.collections WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(collection.0)
    }

    async fn update_collection(&self, collection: &Collection) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE meta.collections SET collection = $2 WHERE id = $1")
            .bind(&collection.id)
            .bind(Json(collection))
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_collection(&self, id: &str) -> Result<(), anyhow::Error> {
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

    async fn list_collections(&self, _query: &Query) -> Result<Collections, anyhow::Error> {
        let collections = sqlx::query_scalar!(
            r#"
            SELECT array_to_json(array_agg(collection)) as "collections: sqlx::types::Json<Vec<Collection>>" 
            FROM meta.collections
            "#)
            .fetch_one(&self.pool)
            .await?;

        let collections = collections.map(|c| c.0).unwrap_or_default();
        let mut collections = Collections::new(collections);
        collections.number_matched = collections.number_returned;

        Ok(collections)
    }
}

use sqlx::types::Json;

use ogcapi_types::common::{Collection, Collections, Crs, Query};

use crate::CollectionTransactions;

use super::Db;

const COLLECTION: &str = r#"
CASE
    WHEN (
        collection #> '{{extent,spatial}}' IS NULL 
        AND ST_EstimatedExtent('items', collection ->> 'id', 'geom') IS NOT NULL
    )
    THEN collection || (SELECT
        jsonb_build_object('extent',
        jsonb_build_object('spatial',
        jsonb_build_object(
            'bbox',
            (
                WITH extent AS (
                    SELECT ST_EstimatedExtent('items', collection ->> 'id', 'geom') AS e
                )
                SELECT ARRAY[ARRAY[ST_XMin(e), ST_YMin(e), ST_XMax(e), ST_YMax(e)]]
                FROM extent
            ),
            'crs',
            collection -> 'storageCrs'
        )))
    )
    ELSE collection
END
"#;

#[async_trait::async_trait]
impl CollectionTransactions for Db {
    async fn create_collection(&self, collection: &Collection) -> anyhow::Result<String> {
        let srid = collection
            .storage_crs
            .as_ref()
            .map(|crs| crs.as_srid())
            .unwrap_or_else(|| Crs::default2d().as_srid());

        let mut tx = self.pool.begin().await?;

        sqlx::query(&format!(
            r#"
            CREATE TABLE items."{0}" (
                id text PRIMARY KEY DEFAULT gen_random_uuid()::text,
                collection text REFERENCES meta.collections(id) DEFAULT '{0}',
                properties jsonb,
                geom geometry(GEOMETRY, {srid}) NOT NULL,
                links jsonb NOT NULL DEFAULT '[]'::jsonb,
                assets jsonb NOT NULL DEFAULT '{{}}'::jsonb,
                bbox jsonb
            )
            "#,
            collection.id
        ))
        .execute(&mut *tx)
        .await?;

        sqlx::query(&format!(
            r#"CREATE INDEX ON items."{}" USING btree (collection)"#,
            collection.id
        ))
        .execute(&mut *tx)
        .await?;

        sqlx::query(&format!(
            r#"CREATE INDEX ON items."{}" USING gin (properties)"#,
            collection.id
        ))
        .execute(&mut *tx)
        .await?;

        sqlx::query(&format!(
            r#"CREATE INDEX ON items."{}" USING gist (geom)"#,
            collection.id
        ))
        .execute(&mut *tx)
        .await?;

        sqlx::query("INSERT INTO meta.collections ( id, collection ) VALUES ( $1, $2 )")
            .bind(&collection.id)
            .bind(Json(collection))
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(collection.id.to_owned())
    }

    async fn read_collection(&self, collection_id: &str) -> anyhow::Result<Option<Collection>> {
        // TODO: cache
        let collection: Option<Json<Collection>> = sqlx::query_scalar(&format!(
            r#"
            SELECT {COLLECTION} AS "collection!"
            FROM meta.collections
            WHERE id = $1
            "#,
        ))
        .bind(collection_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(collection.map(|c| c.0))
    }

    async fn update_collection(&self, collection: &Collection) -> anyhow::Result<()> {
        sqlx::query("UPDATE meta.collections SET collection = $2 WHERE id = $1")
            .bind(&collection.id)
            .bind(Json(collection))
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_collection(&self, collection_id: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(&format!(r#"DROP TABLE IF EXISTS items."{collection_id}""#))
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM meta.collections WHERE id = $1")
            .bind(collection_id)
            .fetch_optional(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn list_collections(&self, _query: &Query) -> anyhow::Result<Collections> {
        let where_clause = match cfg!(feature = "stac") {
            true => Some(r#"WHERE collection ->> 'type' = 'Collection'"#),
            false => None,
        };
        let collections: Option<Json<Vec<Collection>>> = sqlx::query_scalar(&format!(
            "SELECT array_to_json(array_agg({COLLECTION})) FROM meta.collections {}",
            where_clause.unwrap_or_default()
        ))
        .fetch_one(&self.pool)
        .await?;

        let collections = collections.map(|c| c.0).unwrap_or_default();
        let mut collections = Collections::new(collections);
        collections.number_matched = collections.number_returned;

        Ok(collections)
    }
}

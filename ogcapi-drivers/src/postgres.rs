use std::convert::TryInto;

use sqlx::{
    postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions},
    types::Json,
    Connection, Executor,
};
use url::Url;

use ogcapi_types::common::Collection;
use ogcapi_types::features::Feature;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: PgPool,
}

impl Db {
    /// Create
    pub async fn setup(url: &Url) -> Result<Self, anyhow::Error> {
        let name = url.path().strip_prefix('/').unwrap();
        Db::setup_with(url, name, false).await
    }

    pub async fn setup_with(url: &Url, name: &str, create: bool) -> Result<Self, anyhow::Error> {
        // Connection options
        let mut options = PgConnectOptions::new_without_pgpass();
        if url.has_host() {
            options = options.host(url.host_str().unwrap())
        }
        if let Some(port) = url.port() {
            options = options.password(&port.to_string());
        }
        if !url.username().is_empty() {
            options = options.username(url.username())
        }
        if let Some(password) = url.password() {
            options = options.password(password);
        }

        if create {
            // Create database
            let mut connection = PgConnection::connect_with(&options)
                .await
                .expect("Failed to connect to Postgres");
            connection
                .execute(format!(r#"CREATE DATABASE "{}";"#, name).as_str())
                .await
                .expect("Failed to create database.");
        }

        // Create pool
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .connect_with(options.database(name))
            .await
            .expect("Failed to connect to Postgres.");

        // This embeds database migrations in the application binary so we can
        // ensure the database is migrated correctly on startup
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Failed to migrate the database");

        Ok(Db { pool })
    }

    pub async fn insert_collection(
        &self,
        collection: &Collection,
    ) -> Result<String, anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS items.{} (
                id bigserial PRIMARY KEY,
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

        Ok(format!("collections/{}", collection.id))
    }

    pub async fn select_collection(&self, id: &str) -> Result<Collection, anyhow::Error> {
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

    pub async fn update_collection(&self, collection: &Collection) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE meta.collections SET collection = $2 WHERE id = $1")
            .bind(&collection.id)
            .bind(Json(collection))
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

    pub async fn select_feature(
        &self,
        collection: &str,
        id: &i64,
        crs: Option<i32>,
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
        .bind(crs.unwrap_or(4326))
        .fetch_one(&self.pool)
        .await?;

        Ok(feature.0)
    }

    pub async fn update_feature(&self, feature: &Feature) -> Result<(), anyhow::Error> {
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

    pub async fn delete_feature(&self, collection: &str, id: &i64) -> Result<(), anyhow::Error> {
        sqlx::query(&format!("DELETE FROM items.{} WHERE id = $1", collection))
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn storage_srid(&self, collection: &str) -> Result<String, anyhow::Error> {
        let row: (String,) = sqlx::query_as(
            "SELECT collection ->> 'storageCrs' FROM meta.collections WHERE id = $1",
        )
        .bind(&collection)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0.split('/').last().unwrap().to_string())
    }
}

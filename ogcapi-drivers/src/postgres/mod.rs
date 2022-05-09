mod collection;
mod edr;
mod feature;
mod job;
mod style;
mod tile;

use sqlx::{
    postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions},
    Connection, Executor,
};
use url::Url;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: PgPool,
}

impl Db {
    /// Create Postgres Driver
    pub async fn setup(url: &Url) -> Result<Self, sqlx::Error> {
        let name = url.path().strip_prefix('/').unwrap();
        Db::setup_with(url, name, false).await
    }

    pub async fn setup_with(url: &Url, name: &str, create: bool) -> Result<Self, sqlx::Error> {
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

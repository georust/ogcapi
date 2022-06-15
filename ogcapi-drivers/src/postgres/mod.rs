mod collection;
mod edr;
mod feature;
mod job;
mod style;
mod tile;

use sqlx::{
    migrate::MigrateDatabase,
    postgres::{PgPool, PgPoolOptions},
    Postgres,
};
use url::Url;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: PgPool,
}

impl Db {
    /// Setup database driver
    pub async fn setup(url: &Url) -> Result<Self, sqlx::Error> {
        // Create database if not exists
        if !Postgres::database_exists(url.as_str()).await? {
            Postgres::create_database(url.as_str()).await?
        }

        // Create pool
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .connect(url.as_str())
            .await?;

        // This embeds database migrations in the application binary so we can
        // ensure the database is migrated correctly on startup
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Failed to migrate the database");

        Ok(Db { pool })
    }

    pub async fn storage_srid(&self, collection: &str) -> Result<i32, anyhow::Error> {
        let srid = sqlx::query_scalar!(
            r#"
            SELECT srid 
            FROM public.geometry_columns 
            WHERE f_table_schema = 'items' AND f_table_name = $1 AND f_geometry_column = 'geom'
            "#,
            &collection
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(srid.expect(&format!(
            "Geometry column `geom` of table `items.{collection}` has no srid set!"
        )))
    }
}

#[cfg(feature = "common")]
mod collection;
#[cfg(feature = "edr")]
mod edr;
#[cfg(feature = "features")]
mod feature;
#[cfg(feature = "processes")]
mod job;
#[cfg(feature = "stac")]
mod stac;
#[cfg(feature = "styles")]
mod style;
#[cfg(feature = "tiles")]
mod tile;

use sqlx::{
    migrate::MigrateDatabase,
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    Postgres,
};
use url::Url;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: PgPool,
}

impl Db {
    /// Create driver from env `DATABASE_URL` or else `PGUSER` and friends
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = if let Ok(url) = std::env::var("DATABASE_URL") {
            PgPoolOptions::new()
                .max_connections(8)
                .connect(&url)
                .await?
        } else {
            PgPoolOptions::new()
                .max_connections(8)
                .connect_with(PgConnectOptions::new())
                .await?
        };

        Ok(Db { pool })
    }

    /// Setup database driver from url
    pub async fn setup(url: &Url) -> Result<Self, sqlx::Error> {
        // Create database if not exists
        if !Postgres::database_exists(url.as_str()).await? {
            Postgres::create_database(url.as_str()).await?
        }

        // Create pool
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(url.as_str())
            .await?;

        // Run embedded migrations
        sqlx::migrate!().run(&pool).await?;

        Ok(Db { pool })
    }
}

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
    Postgres,
    migrate::MigrateDatabase,
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
};
use url::Url;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: PgPool,
}

/// The table of a collection's items, `items."<id>"`, or `None` for an id
/// that no collection can have.
///
/// A collection's id goes into SQL as an identifier, and as a literal in its
/// table's definition and a tile's layer name, so an id holding a quote, a
/// backslash or a control character is refused, and so is one longer than the
/// 63 bytes of an identifier, which Postgres would cut short, giving two ids
/// one table.
#[cfg(feature = "common")]
fn items_table(collection_id: &str) -> Option<String> {
    let valid = !collection_id.is_empty()
        && collection_id.len() <= 63
        && !collection_id
            .chars()
            .any(|c| c.is_control() || matches!(c, '"' | '\'' | '\\'));

    valid.then(|| format!(r#"items."{collection_id}""#))
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
            Postgres::create_database(url.as_str()).await?;
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

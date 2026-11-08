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

#[cfg(any(feature = "features", feature = "edr"))]
use anyhow::Context;
use sqlx::{
    Postgres,
    migrate::MigrateDatabase,
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
};
use url::Url;

#[cfg(any(feature = "features", feature = "edr"))]
use ogcapi_types::common::Collection;
#[cfg(feature = "features")]
use ogcapi_types::common::{Bbox, Datetime, IntervalDatetime};

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

/// The envelopes of a box's horizontal extent in `srid`: two for longitudes
/// across the antimeridian (its west east of its east) where `srid` is
/// geographic (4326, 4979).
#[cfg(feature = "features")]
fn envelopes(bbox: &Bbox, srid: i32) -> Vec<String> {
    let [west, south, east, north] = match *bbox {
        Bbox::Bbox2D(bbox) => bbox,
        Bbox::Bbox3D([west, south, _, east, north, _]) => [west, south, east, north],
    };
    let envelope =
        |west: f64, east: f64| format!("ST_MakeEnvelope({west}, {south}, {east}, {north}, {srid})");
    if west > east && matches!(srid, 4326 | 4979) {
        vec![envelope(west, 180.0), envelope(-180.0, east)]
    } else {
        vec![envelope(west, east)]
    }
}

/// The condition that a feature's `datetime`, or its `start_datetime` to
/// `end_datetime`, meets `datetime`, an open end unbounded; a feature
/// without a time meets any.
#[cfg(feature = "features")]
fn datetime_condition(datetime: &Datetime) -> String {
    let instant = |datetime: &dyn std::fmt::Display| format!("CAST('{datetime}' AS timestamptz)");
    let (from, to) = match datetime {
        Datetime::Datetime(_) => (instant(datetime), instant(datetime)),
        Datetime::Interval { from, to } => {
            let end = |end: &IntervalDatetime, open: &str| match end {
                IntervalDatetime::Datetime(_) => instant(end),
                IntervalDatetime::Open => format!("'{open}'::timestamptz"),
            };
            (end(from, "-infinity"), end(to, "infinity"))
        }
    };
    format!(
        r"
        (
            CASE
                WHEN (properties->'datetime') IS NOT NULL THEN (
                    CAST(properties->>'datetime' AS timestamptz)
                    BETWEEN {from} AND {to}
                )
                WHEN (
                    (properties->'datetime') IS NULL
                    AND (properties->'start_datetime') IS NOT NULL
                    AND (properties->'end_datetime') IS NOT NULL
                ) THEN (
                    ({from}, {to}) OVERLAPS (
                        CAST(properties->>'start_datetime' AS timestamptz),
                        CAST(properties->>'end_datetime' AS timestamptz)
                    )
                )
                ELSE TRUE
            END
        )
        "
    )
}

impl Db {
    /// The SRID of a collection's geometries: its `storageCrs`'s, else its
    /// column's.
    #[cfg(any(feature = "features", feature = "edr"))]
    async fn storage_srid(&self, collection: &Collection) -> anyhow::Result<i32> {
        match &collection.storage_crs {
            Some(crs) => crs.as_srid().context("cannot determine SRID"),
            None => Ok(sqlx::query_scalar("SELECT Find_SRID('items', $1, 'geom')")
                .bind(&collection.id)
                .fetch_one(&self.pool)
                .await?),
        }
    }

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

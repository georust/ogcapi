use std::env;

use sqlx::postgres::PgRow;
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, types::Json, Pool, Postgres};

use crate::collections::{Collection, Extent, ItemType, Provider, Summaries};
use crate::common::{Conformance, Link};

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Pool<Postgres>,
}

impl Db {
    pub async fn connect() -> Result<Self, anyhow::Error> {
        let url = env::var("DATABASE_URL")?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;

        Ok(Db { pool })
    }

    pub async fn root(&self) -> Result<Vec<Link>, anyhow::Error> {
        let links = sqlx::query("SELECT row_to_json(root) FROM root")
            .try_map(|row: PgRow| {
                serde_json::from_value::<Link>(row.get(0))
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(links)
    }

    pub async fn conformance(&self) -> Result<Conformance, anyhow::Error> {
        let classes = sqlx::query_scalar!("SELECT * FROM conformance")
            .fetch_all(&self.pool)
            .await?;

        Ok(Conformance {
            conforms_to: classes,
        })
    }

    pub async fn create_collection(
        &self,
        collection: &Collection,
    ) -> Result<Collection, anyhow::Error> {
        let collection = sqlx::query_file_as!(
            Collection,
            "sql/collection_insert.sql",
            collection.id,
            collection.title,
            collection.description,
            collection.links as _,
            collection.extent as _,
            collection.item_type as _,
            collection.crs.as_deref(),
            collection.storage_crs,
            collection.storage_crs_coordinate_epoch,
            collection.stac_version,
            collection.stac_extensions.as_deref(),
            collection.keywords.as_deref(),
            collection.licence,
            collection.providers as _,
            collection.summaries as _
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(collection)
    }

    pub async fn read_collection(&self, id: &str) -> Result<Collection, anyhow::Error> {
        let collection: Collection =
            sqlx::query_file_as!(Collection, "sql/collection_select.sql", id)
                .fetch_one(&self.pool)
                .await?;

        Ok(collection)
    }

    pub async fn update_collection(
        &self,
        collection: &Collection,
    ) -> Result<Collection, anyhow::Error> {
        let collection = sqlx::query_file_as!(
            Collection,
            "sql/collection_update.sql",
            collection.id,
            collection.title,
            collection.description,
            collection.links as _,
            collection.extent as _,
            collection.item_type as _,
            collection.crs.as_deref(),
            collection.storage_crs,
            collection.storage_crs_coordinate_epoch,
            collection.stac_version,
            collection.stac_extensions.as_deref(),
            collection.keywords.as_deref(),
            collection.licence,
            collection.providers as _,
            collection.summaries as _
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(collection)
    }

    pub async fn delete_collection(&self, id: &str) -> Result<(), anyhow::Error> {
        sqlx::query_file_as!(Collection, "sql/collection_delete.sql", id)
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }
}

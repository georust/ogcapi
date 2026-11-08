use anyhow::Context;
use ogcapi_types::{common::Exception, tiles::TileMatrixSet};

use crate::{CollectionTransactions, TileTransactions};

use super::{Db, items_table};

#[async_trait::async_trait]
impl TileTransactions for Db {
    async fn tile(
        &self,
        collections: &[String],
        _tms: &TileMatrixSet,
        matrix: &str,
        row: u32,
        col: u32,
    ) -> anyhow::Result<Vec<u8>> {
        let mut sql: Vec<String> = Vec::new();

        for collection_id in collections {
            // the id, valid as a table's, is valid as the layer's name too
            let Some(table) = items_table(collection_id) else {
                continue;
            };
            if let Some(collection) = self.read_collection(collection_id).await? {
                let storage_srid = match collection.storage_crs.map(|crs| crs.as_srid()) {
                    Some(srid) => srid.context("cannot determine SRID")?,
                    None => {
                        sqlx::query_scalar("SELECT Find_SRID('items', $1, 'geom')")
                            .bind(collection_id)
                            .fetch_one(&self.pool)
                            .await?
                    }
                };

                sql.push(format!(
                    r"
                    SELECT ST_AsMVT(mvtgeom, '{collection_id}', 4096, 'geom')
                    FROM (
                        SELECT
                            ST_AsMVTGeom(ST_Transform(ST_Force2D(geom), 3857), ST_TileEnvelope($1, $3, $2), 4096, 64, TRUE) AS geom,
                            '{collection_id}' as collection,
                            properties
                        FROM {table}
                        WHERE geom && ST_Transform(ST_TileEnvelope($1, $3, $2, margin => (64.0 / 4096)), {storage_srid})
                    ) AS mvtgeom
                    "
                ));
            }
        }

        if sql.is_empty() {
            return Err(Exception::new_from_status(404).into());
        }

        let tiles: Vec<Vec<u8>> = sqlx::query_scalar(&sql.join(" UNION ALL "))
            .bind(matrix.parse::<i32>()?)
            .bind(row.cast_signed())
            .bind(col.cast_signed())
            .fetch_all(&self.pool)
            .await?;

        Ok(tiles.concat())
    }
}

use ogcapi_types::tiles::TileMatrixSet;

use crate::{CollectionTransactions, TileTransactions};

use super::Db;

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

        for collection in collections {
            if let Some(c) = self.read_collection(collection).await? {
                let storage_srid = c.storage_crs.unwrap_or_default().as_srid();

                sql.push(format!(
                    r#"
                    SELECT ST_AsMVT(mvtgeom, '{collection}', 4096, 'geom')
                    FROM (
                        SELECT
                            ST_AsMVTGeom(ST_Transform(ST_Force2D(geom), 3857), ST_TileEnvelope($1, $3, $2), 4096, 64, TRUE) AS geom,
                            '{collection}' as collection,
                            properties
                        FROM items.{collection}
                        WHERE geom && ST_Transform(ST_TileEnvelope($1, $3, $2, margin => (64.0 / 4096)), {storage_srid})
                    ) AS mvtgeom
                    "#
                ));
            };
        }

        let tiles: Vec<Vec<u8>> = sqlx::query_scalar(&sql.join(" UNION ALL "))
            .bind(matrix.parse::<i32>().unwrap())
            .bind(row as i32)
            .bind(col as i32)
            .fetch_all(&self.pool)
            .await?;

        Ok(tiles.concat())
    }
}

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

        for collection_id in collections {
            if let Some(collection) = self.read_collection(collection_id).await? {
                let storage_srid = match collection.storage_crs.map(|crs| crs.as_srid()) {
                    Some(srid) => srid,
                    None => {
                        sqlx::query_scalar(&format!(
                            "SELECT Find_SRID('items', '{collection_id}', 'geom')"
                        ))
                        .fetch_one(&self.pool)
                        .await?
                    }
                };

                sql.push(format!(
                    r#"
                    SELECT ST_AsMVT(mvtgeom, '{collection_id}', 4096, 'geom')
                    FROM (
                        SELECT
                            ST_AsMVTGeom(ST_Transform(ST_Force2D(geom), 3857), ST_TileEnvelope($1, $3, $2), 4096, 64, TRUE) AS geom,
                            '{collection_id}' as collection,
                            properties
                        FROM items.{collection_id}
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

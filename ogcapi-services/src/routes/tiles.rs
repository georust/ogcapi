use axum::extract::{Extension, Path};
use axum::routing::get;
use axum::Router;

use crate::{Result, State};
// use crate::tiles::{TileMatrixSet, TileMatrixSets, TileSet};

// async fn tile_matrix_sets() -> Result<Json<TileMatrixSets>> {
//     let tile_matrix_sets = TileMatrixSets {
//         tile_matrix_sets: vec![],
//     };

//     Ok(Json(tile_matrix_sets))
// }

// async fn tile_matrix_set(Path(id): Path<String>) -> Result<Json<TileMatrixSet>> {
//     let tile_matrix_set;
//     Ok(Json(tile_matrix_set))
// }

// async fn tiles() -> Result<Json<TileSet>> {
//     let tile_set;
//     Ok(Json(tille_set))
// }

async fn tile(
    Path((collections, _matrix_set_id, matrix, row, col)): Path<(String, String, i32, i32, i32)>,
    Extension(state): Extension<State>,
) -> Result<Vec<u8>> {
    let collections: Vec<&str> = collections.split(',').collect();

    let mut sql: Vec<String> = Vec::new();

    for collection in collections {
        let srid = state.db.storage_srid(collection).await?;

        sql.push(format!(
            r#"
            SELECT ST_AsMVT(mvtgeom, '{0}', 4096, 'geom', 'id')
            FROM (
                SELECT
                    ST_AsMVTGeom(ST_Transform(ST_Force2D(geom), 3857), ST_TileEnvelope($1, $3, $2), 4096, 64, TRUE) AS geom,
                    '{0}' as collection,
                    id,
                    properties
                FROM items.{0}
                WHERE geom && ST_Transform(ST_TileEnvelope($1, $3, $2, margin => (64.0 / 4096)), {1})
            ) AS mvtgeom
            "#,
            collection,
            srid
        ));
    }

    let tiles: Vec<Vec<u8>> = sqlx::query_scalar(&sql.join(" UNION ALL "))
        .bind(matrix)
        .bind(row)
        .bind(col)
        .fetch_all(&state.db.pool)
        .await?;

    Ok(tiles.concat())
}

pub(crate) fn router(_state: &State) -> Router {
    Router::new()
        // .route("/tileMatrixSets", get(tile_matrix_sets))
        // .route("/tileMatrixSets/:id", get(tile_matrix_set))
        // .route("collections/:collectionId/tiles", get(tiles))
        .route(
            "collections/:collectionId/tiles/:tileMatrixSetId/:tileMatrix/:tileRow/:tileCol",
            get(tile),
        )
}

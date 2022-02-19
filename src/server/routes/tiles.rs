use tide::{Body, Request, Response, Result, Server};

use crate::server::State;
use crate::tiles::{Query, TileMatrixSets};

async fn tile_matrix_sets(_req: Request<State>) -> Result {
    let tile_matrix_sets = TileMatrixSets {
        tile_matrix_sets: vec![],
    };
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&tile_matrix_sets)?);
    Ok(res)
}

async fn tile_matrix_set(req: Request<State>) -> Result {
    let _id = req.param("tileMatrixSetId")?;
    let res = Response::new(200);
    // res.set_body(Body::from_json(&matrix_set)?);
    Ok(res)
}

async fn tiles(_req: Request<State>) -> Result {
    // let tile_set = TileSet {};
    let res = Response::new(200);
    // res.set_body(Body::from_json(&tile_set)?);
    Ok(res)
}

async fn tile(req: Request<State>) -> Result {
    let _matrix_set_id = req.param("tileMatrixSetId")?;
    let matrix: i32 = req.param("tileMatrix")?.parse()?; // zoom, z
    let row: i32 = req.param("tileRow")?.parse()?; // x
    let col: i32 = req.param("tileCol")?.parse()?; // y

    let _query: Query = req.query()?;

    let collections: Vec<String> = req
        .param("collectionId")?
        .split(',')
        .map(str::to_owned)
        .collect();

    let mut sql: Vec<String> = Vec::new();

    for collection in collections.clone() {
        let srid = req.state().db.storage_srid(&collection).await?;

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
        .fetch_all(&req.state().db.pool)
        .await?;

    let mut res = Response::new(200);
    res.set_body(tiles.concat());
    Ok(res)
}

pub(crate) fn register(app: &mut Server<State>) {
    app.at("tileMatrixSets").get(tile_matrix_sets);
    app.at("tileMatrixSets/:tileMatrixSetId")
        .get(tile_matrix_set);
    app.at("collections/:collectionId/tiles").get(tiles);
    app.at("collections/:collectionId/tiles/:tileMatrixSetId/:tileMatrix/:tileRow/:tileCol")
        .get(tile);
}

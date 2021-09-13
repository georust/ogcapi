use tide::{Request, Response, Result};

use crate::db::Db;
use crate::tiles::Query;

/*
pub async fn get_tile_matrix_sets(req: Request<Service>) -> Result {
    let tile_matrix_sets = TileMatrixSets {
        tile_matrix_sets: vec![],
    };
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&tile_matrix_sets)?);
    Ok(res)
}


pub async fn get_tile_matrix_set(req: Request<Service>) -> Result {
    let tile_matrix_set = TileMatrixSet {};
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&tile_matrix_set)?);
    Ok(res)
}


pub async fn hadle_tiles(req: Request<Service>) -> Result {
    let tile_set = TileSet {
    };
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&tile_set)?);
    Ok(res)
}
*/

pub async fn get_tile(req: Request<Db>) -> Result {
    let _matrix_set = req.param("matrix_set")?;
    let matrix: i32 = req.param("matrix")?.parse()?; // zoom, z
    let row: i32 = req.param("row")?.parse()?; // x
    let col: i32 = req.param("col")?.parse()?; // y

    let query: Query = req.query()?;

    let collections: Vec<String> = req
        .param("collection")
        .ok()
        .map(|c| c.to_owned())
        .unwrap_or_else(|| query.collections.unwrap())
        .split(",")
        .map(|c| c.to_owned())
        .collect();

    let sql = collections.iter().map(|c| format!(r#"
    SELECT ST_AsMVT(mvtgeom, '{0}', 4096, 'geom', 'id')
    FROM (
        SELECT ST_AsMVTGeom(ST_Transform(ST_Force2D(geom), 3857), ST_TileEnvelope($1, $3, $2), 4096, 64, TRUE) AS geom, '{0}' as collection, id, properties
        FROM items.{0}
        WHERE geom && ST_Transform(ST_TileEnvelope($1, $3, $2, margin => (64.0 / 4096)), 4326)
    ) AS mvtgeom
    "#, c)).collect::<Vec<String>>().join(" UNION ");

    let tiles: Vec<Vec<u8>> = sqlx::query_scalar(&sql)
        .bind(matrix)
        .bind(row)
        .bind(col)
        .fetch_all(&req.state().pool)
        .await?;

    let mut res = Response::new(200);
    res.set_body(tiles.concat());
    Ok(res)
}

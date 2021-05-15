use tide::{Request, Response, Result};

use crate::tiles::Tile;

use super::Service;

// pub async fn get_tile_matrix_sets(req: Request<Service>) -> Result {
//     let tile_matrix_sets = TileMatrixSets {
//         tile_matrix_sets: vec![],
//     };
//     let mut res = Response::new(200);
//     res.set_body(Body::from_json(&tile_matrix_sets)?);
//     Ok(res)
// }

// pub async fn get_tile_matrix_set(req: Request<Service>) -> Result {
//     let tile_matrix_set = TileMatrixSet {};
//     let mut res = Response::new(200);
//     res.set_body(Body::from_json(&tile_matrix_set)?);
//     Ok(res)
// }

// pub async fn hadle_tiles(req: Request<Service>) -> Result {
//     let tile_set = TileSet {

//     };
//     let mut res = Response::new(200);
//     res.set_body(Body::from_json(&tile_set)?);
//     Ok(res)
// }

pub async fn get_tile(req: Request<Service>) -> Result {
    let collection = req.param("collection")?;
    let _matrix_set = req.param("matrix_set")?;
    let matrix: i32 = req.param("matrix")?.parse()?; // zoom, z
    let row: i32 = req.param("row")?.parse()?; // x
    let col: i32 = req.param("col")?.parse()?; // y

    let tile = sqlx::query_file_as!(Tile, "sql/tile_select.sql", collection, matrix, row, col)
        .fetch_one(&req.state().pool)
        .await?;

    let mut res = Response::new(200);
    res.set_body(tile.st_asmvt.unwrap());
    Ok(res)
}

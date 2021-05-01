mod routes;

pub use self::routes::*;

use crate::common::Link;
use serde::Serialize;
use tide::http::Url;

#[serde(rename_all = "camelCase")]
#[derive(Serialize)]
pub struct TileMatrixSets {
    pub tile_matrix_sets: Vec<IdLink>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize)]
pub struct IdLink {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub links: Vec<Link>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize)]
pub struct TileSet {
    pub tile_set: Vec<TileSetEntry>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize)]
pub struct TileSetEntry {
    pub tile_url: Url,
    pub tile_matrix: String,
    pub tile_row: Option<i32>,
    pub tile_col: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub top: Option<i32>,
    pub left: Option<i32>,
}

struct Tile {
    st_asmvt: Option<Vec<u8>>,
}

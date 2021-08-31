use serde::{Deserialize, Serialize};
use tide::http::Url;

use crate::common::Links;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrixSets {
    pub tile_matrix_sets: Vec<IdLink>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdLink {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub links: Links,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSet {
    pub tile_set: Vec<TileSetEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Deserialize)]
pub struct Query {
    pub collections: Option<String>,
}

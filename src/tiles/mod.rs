mod meta;
mod tileset;
mod tms;

use tms::TileMatrixSet;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::common::core::Links;

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
    pub tile_url: String,
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

#[derive(Serialize, Deserialize)]
enum Crs {
    /// Simplification of the object into a url if the other properties are not
    /// present
    Url(String),
    Uri {
        /// Reference to one coordinate reference system (CRS)
        uri: String,
    },
    Wkt {
        /// A string defining the CRS using the JSON encodng for Well Known Text
        wkt: String,
    },
    #[serde(rename_all = "camelCase")]
    ReferenceSystem {
        /// A reference system data structure as defined in the
        /// MD_ReferenceSystem of the ISO 19115
        reference_system: Map<String, Value>,
    },
}

/// Minimum bounding rectangle surrounding a 2D resource in the CRS indicated
/// elsewere
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BoundingBox2D {
    lower_left: Point2D,
    upper_right: Point2D,
    crs: Option<Crs>,
    orderd_axes: Option<OrderedAxes>,
}

#[derive(Serialize, Deserialize)]
struct Point2D(f64, f64);

#[derive(Serialize, Deserialize)]
struct OrderedAxes(String, String);

pub use tileset::*;
pub use tms::*;

mod tileset;
mod tms;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::DisplayFromStr;
use utoipa::{IntoParams, ToSchema};

use crate::common::Crs;

/// A 2DPoint in the CRS indicated elsewere
type Point2D = [f64; 2];

/// Coordinate Reference System (CRS)
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum TilesCrs {
    /// Simplification of the object into a url if the other properties are not present
    #[schema(value_type = String)]
    Simple(Crs),
    /// Reference to one coordinate reference system (CRS)
    Uri { uri: String },
    /// An object defining the CRS using the JSON encoding for Well-known text
    /// representation of coordinate reference systems 2.0
    Wkt { wkt: Map<String, Value> },
    /// A reference system data structure as defined in the MD_ReferenceSystem of the ISO 19115
    ReferenceSystem {
        #[serde(rename = "referenceSystem")]
        reference_system: String,
    },
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TileParams {
    /// Identifier selecting one of the TileMatrixSetId supported by the resource.
    pub tile_matrix_set_id: TileMatrixSetId,
    /// Identifier selecting one of the scales defined in the TileMatrixSet
    /// and representing the scaleDenominator the tile.
    pub tile_matrix: String,
    /// Row index of the tile on the selected TileMatrix. It cannot exceed
    /// the MatrixWidth-1 for the selected TileMatrix.
    #[serde_as(as = "DisplayFromStr")]
    pub tile_row: u32,
    /// Column index of the tile on the selected TileMatrix. It cannot exceed
    /// the MatrixHeight-1 for the selected TileMatrix.
    #[serde_as(as = "DisplayFromStr")]
    pub tile_col: u32,
}

#[derive(Serialize, Deserialize, IntoParams, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionTileParams {
    /// Local identifier of a vector tile collection
    pub collection_id: String,
    #[serde(flatten)]
    pub tile_params: TileParams,
}

#[derive(Serialize, Deserialize, IntoParams, Debug)]
#[serde(rename_all = "kebab-case")]
#[into_params(parameter_in = Query)]
pub struct TileQuery {
    // /// Either a date-time or an interval, half-bounded or bounded. Date and
    // /// time expressions adhere to RFC 3339. Half-bounded intervals are
    // /// expressed using double-dots.
    // ///
    // /// Examples:
    // ///
    // /// * A date-time: "2018-02-12T23:20:50Z"
    // /// * A bounded interval: "2018-02-12T00:00:00Z/2018-03-18T12:31:12Z"
    // /// * Half-bounded intervals: "2018-02-12T00:00:00Z/.." or "../2018-03-18T12:31:12Z"
    // ///
    // /// Only features that have a temporal property that intersects the value
    // /// of datetime are selected.
    // ///
    // /// If a feature has multiple temporal properties, it is the decision of
    // /// the server whether only a single temporal property is used to determine
    // /// the extent or all relevant temporal properties.
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // #[serde_as(as = "Option<DisplayFromStr>")]
    // #[param(value_type = String, nullable = false)]
    // pub datetime: Option<Datetime>,
    /// The collections that should be included in the response. The parameter
    /// value is a comma-separated list of collection identifiers. If the
    /// parameters is missing, some or all collections will be included. The
    /// collection will be rendered in the order specified, with the last one
    /// showing on top, unless the priority is overridden by styling rules.
    #[serde(
        default,
        with = "serde_qs::helpers::comma_separated",
        skip_serializing_if = "Vec::is_empty"
    )]
    #[param(style = Form, explode = false, required = false)]
    pub collections: Vec<String>,
    // /// Retrieve only part of the data by slicing or trimming along one or more
    // /// axis For trimming: {axisAbbrev}({low}:{high}) (preserves dimensionality)
    // /// An asterisk (*) can be used instead of {low} or {high} to indicate the
    // /// minimum/maximum value. For slicing: {axisAbbrev}({value}) (reduces dimensionality)
    // #[serde(
    //     default,
    //     with = "serde_qs::helpers::comma_separated",
    //     skip_serializing_if = "Vec::is_empty"
    // )]
    // #[param(style = Form, explode = false, required = false)]
    // pub subset: Vec<String>,
    // /// reproject the output to the given crs
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub crs: Option<String>,
    // /// crs for the specified subset
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub subset_crs: Option<String>,
}

/// Minimum bounding rectangle surrounding a 2D resource in the CRS indicated elsewere
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBox2D {
    /// A 2DPoint in the CRS indicated elsewere
    #[schema(value_type = Vec<f64>, min_items = 2, max_items = 2, inline = true)]
    pub lower_left: Point2D,
    #[schema(value_type = Vec<f64>, min_items = 2, max_items = 2, inline = true)]
    pub upper_right: Point2D,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub crs: Option<TilesCrs>,
    /// Ordered list of names of the dimensions defined in the CRS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Vec<String>, min_items = 2, max_items = 2, nullable = false, inline = true)]
    pub ordered_axes: Option<[String; 2]>,
}

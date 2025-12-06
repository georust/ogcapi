use serde::{Deserialize, Serialize};
use std::num::{NonZeroU16, NonZeroU64};
use utoipa::ToSchema;

use crate::common::Link;

use super::{BoundingBox2D, Point2D, TilesCrs};

/// Identifier for a supported TileMatrixSet
#[derive(
    Serialize, Deserialize, ToSchema, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum TileMatrixSetId {
    #[default]
    WebMercatorQuad,
    // WorldCRS84Quad,
    // GNOSISGlobalGrid,
    // WorldMercatorWGS84Quad,
}

/// A definition of a tile matrix set following the Tile Matrix Set standard.
/// For tileset metadata, such a description (in `tileMatrixSet` property) is
/// only required for offline use, as an alternative to a link with a
/// `http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme` relation type.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrixSets {
    pub tile_matrix_sets: Vec<TileMatrixSetItem>,
}

/// A minimal tile matrix set element for use within a list of tile matrix
/// sets linking to a full definition.
#[derive(Serialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrixSetItem {
    /// Optional local tile matrix set identifier, e.g. for use as unspecified
    /// `{tileMatrixSetId}` parameter. Implementation of 'identifier'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub id: Option<TileMatrixSetId>,
    /// Title of this tile matrix set, normally used for display to a human
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub title: Option<String>,
    /// Reference to an official source for this tileMatrixSet
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub crs: Option<TilesCrs>,
    /// Links to related resources. A 'self' link to the tile matrix set
    /// definition is required.
    pub links: Vec<Link>,
}

/// A definition of a tile matrix set following the Tile Matrix Set standard.
/// For tileset metadata, such a description (in `tileMatrixSet` property) is
/// only required for offline use, as an alternative to a link with a
/// `http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme` relation type.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrixSet {
    /// Tile matrix set identifier
    pub id: TileMatrixSetId,
    /// Title of a tile matrix set, normally used for display to a human
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub title: Option<String>,
    /// Brief narrative description of a tile matrix set, normally available
    /// for display to a human
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub description: Option<String>,
    /// Unordered list of one or more commonly used or formalized word(s) or
    /// phrase(s) used to describe this resource entity
    #[schema(nullable = false)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Reference to an official source for this tile matrix set
    #[schema(nullable = false)]
    pub uri: Option<String>,
    /// Coordinate Reference System (CRS)
    pub crs: TilesCrs,
    /// Ordered list of names of the dimensions defined in the CRS
    #[schema(min_items = 1, inline = true)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ordered_axes: Vec<String>,
    /// Reference to a well-known scale set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub well_known_scale_set: Option<String>,
    /// Minimum bounding rectangle surrounding the tile matrix set, in the
    /// supported CRS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub bounding_box: Option<BoundingBox2D>,
    /// Describes scale levels and its tile matrices
    pub tile_matrices: Vec<TileMatrix>,
}

/// A tile matrix, usually corresponding to a particular zoom level of a
/// TileMatrixSet.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrix {
    /// Identifier selecting one of the scales defined in the [TileMatrixSet]
    /// and representing the scaleDenominator the tile.
    pub id: String,
    /// Title of a tile matrix, normally used for display to a human
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub title: Option<String>,
    /// Brief narrative description of a tile matrix, normally available
    /// for display to a human
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub description: Option<String>,
    /// Unordered list of one or more commonly used or formalized word(s) or
    /// phrase(s) used to describe this tile set
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Scale denominator of this tile matrix
    pub scale_denominator: f64,
    /// Cell size of this tile matrix
    pub cell_size: f64,
    /// The corner of the tile matrix (_topLeft_ or
    /// _bottomLeft_) used as the origin for numbering tile rows and columns.
    /// This corner is also a corner of the (0, 0) tile.
    #[serde(default)]
    pub corner_of_origin: CornerOfOrigin,
    /// Precise position in CRS coordinates of the corner of origin (e.g. the
    /// top-left corner) for this tile matrix. This position is also a corner
    /// of the (0, 0) tile.
    #[schema(value_type = Vec<f64>, min_items = 2, max_items = 2, inline = true)]
    pub point_of_origin: Point2D,
    /// Width of each tile of this tile matrix in pixels
    #[schema(value_type = usize, minimum = 1)]
    pub tile_width: NonZeroU16,
    /// Height of each tile of this tile matrix in pixels
    #[schema(value_type = usize, minimum = 1)]
    pub tile_height: NonZeroU16,
    /// Width of the matrix (number of tiles in width)
    #[schema(value_type = usize, minimum = 1)]
    pub matrix_width: NonZeroU64,
    /// Height of the matrix (number of tiles in height)
    #[schema(value_type = usize, minimum = 1)]
    pub matrix_height: NonZeroU64,
    /// Describes the rows that has variable matrix width
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variable_matrix_widths: Vec<VariableMatrixWidth>,
}

/// Variable Matrix Width data structure
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VariableMatrixWidth {
    /// Number of tiles in width that coalesce in a single tile for these rows
    #[schema(value_type = usize, minimum = 2)]
    pub coalesc: NonZeroU64,
    /// First tile row where the coalescence factor applies for this tilematrix
    pub min_tile_row: u64,
    /// Last tile row where the coalescence factor applies for this tilematrix
    pub smax_tile_row: u64,
}

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CornerOfOrigin {
    #[default]
    TopLeft,
    BottomLeft,
}

#[cfg(test)]
mod test {
    use super::TileMatrixSet;

    #[test]
    fn parse_tms_example() {
        let path = "../ogcapi-services/assets/tms/WebMercartorQuad.json";
        let mut content = std::fs::read_to_string(path).unwrap();
        let tms: TileMatrixSet = serde_json::from_str(&content).unwrap();
        // dbg!(&tms);
        let mut tms_string = serde_json::to_string_pretty(&tms).unwrap();
        // println!("{}", tms_string);

        content.retain(|c| !c.is_whitespace());
        tms_string.retain(|c| !c.is_whitespace());

        assert_eq!(content, tms_string);
    }
}

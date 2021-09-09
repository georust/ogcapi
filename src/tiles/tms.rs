use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use super::{BoundingBox2D, Crs, Point2D};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TileMatrixSet {
    /// Title of this tile matrix set, normally used for display to a human
    title: Option<String>,
    /// Brief narrative description of this tile matrix set, normally available for display to a human
    description: Option<String>,
    /// Unordered list of one or more commonly used or formalized word(s) or
    /// phrase(s) used to describe this tile matrix set
    keywords: Option<Vec<String>>,
    /// Tile matrix set identifier. Implementation of 'identifier'
    id: String,
    /// Reference to an official source for this TileMatrixSet
    uri: Option<String>,
    /// Coordinate Reference System (CRS)
    crs: Crs,
    ordered_axes: Option<Vec<String>>,
    /// Reference to a well-known scale set
    well_known_scale_set: Option<String>,
    /// Minimum bounding rectangle surrounding the tile matrix set, in the
    /// supported CRS
    bounding_box: Option<BoundingBox2D>,
    /// Describes scale levels and its tile matrices
    tile_matrices: Option<TileMatrix>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TileMatrix {
    /// Title of this tile matrix set, normally used for display to a human
    title: Option<String>,
    /// Brief narrative description of this tile matrix set, normally available
    /// for display to a human
    description: Option<String>,
    /// Unordered list of one or more commonly used or formalized word(s) or
    /// phrase(s) used to describe this dataset
    keywords: Option<Vec<String>>,
    /// Identifier selecting one of the scales defined in the [TileMatrixSet]
    /// and representing the scaleDenominator the tile. Implementation of 'identifier'
    id: String,
    /// Scale denominator of this tile matrix
    scale_denominator: u64,
    /// Cell size of this tile matrix
    cell_size: f64,
    /// description": "The corner of the tile matrix (_topLeft_ or
    /// _bottomLeft_) used as the origin for numbering tile rows and columns.
    /// This corner is also a corner of the (0, 0) tile.
    #[serde(default)]
    corner_of_origin: CornerOfOrigin,
    /// Precise position in CRS coordinates of the corner of origin (e.g. the
    /// top-left corner) for this tile matrix. This position is also a corner
    /// of the (0, 0) tile. In previous version, this was 'topLeftCorner' and
    /// 'cornerOfOrigin' did not exist.
    point_of_origin: Point2D,
    /// Width of each tile of this tile matrix in pixels
    tile_width: NonZeroU64,
    /// Height of each tile of this tile matrix in pixels
    tile_height: NonZeroU64,
    /// Width of the matrix (number of tiles in width)
    matrix_width: NonZeroU64,
    /// Height of the matrix (number of tiles in height)
    matrix_height: NonZeroU64,
    /// Describes the rows that has variable matrix width
    variable_matrix_widths: Option<Vec<VariableMatrixWidth>>,
}

/// Variable Matrix Width data structure
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VariableMatrixWidth {
    /// Number of tiles in width that coalesce in a single tile for these rows
    coalesc: NonZeroU64,
    /// First tile row where the coalescence factor applies for this tilematrix
    min_tile_row: u64,
    /// Last tile row where the coalescence factor applies for this tilematrix
    max_tile_row: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CornerOfOrigin {
    TopLeft,
    BottomLeft,
}

impl Default for CornerOfOrigin {
    fn default() -> Self {
        CornerOfOrigin::TopLeft
    }
}

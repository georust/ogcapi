use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::DisplayFromStr;

use crate::common::Links;

use super::{BoundingBox2D, Crs, Point2D, TitleDescriptionKeywords};

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSets {
    pub tilesets: Vec<TileSetItem>,
    pub links: Option<Links>,
}

/// A minimal tileset element for use within a list of tilesets linking to
/// full description of those tilesets.
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSetItem {
    pub title: Option<String>,
    pub data_type: DataType,
    #[serde_as(as = "DisplayFromStr")]
    pub crs: Crs,
    #[serde(rename = "tileMatrixSetURI")]
    pub tile_matrix_set_uri: Option<String>,
    pub links: Links,
}

/// A resource describing a tileset based on the OGC TileSet Metadata Standard.
/// At least one of the 'TileMatrixSet',  or a link with 'rel' tiling-scheme"
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSet {
    #[serde(flatten)]
    title_description_keywords: TitleDescriptionKeywords,
    /// Type of data represented in the tileset
    data_type: DataType,
    /// Reference to a Tile Matrix Set on the OGC NA definition server
    /// (<http://www.opengis.net/def/tms/>). Required if the tile matrix set is
    /// registered on the definition server.
    #[serde(rename = "tileMatrixSetURI")]
    tile_matrix_set_uri: Option<String>,
    /// Limits for the TileRow and TileCol values for each TileMatrix in the
    /// [TileMatrixSet]. If missing, there are no limits other that the ones
    /// imposed by the TileMatrixSet. If present the TileMatrices listed are
    /// limited and the rest not available at all
    tile_matrix_set_limits: Option<Vec<TileMatrixLimits>>,
    /// Coordinate Reference System (CRS)
    crs: Crs,
    /// Epoch of the Coordinate Reference System (CRS)
    epoch: Option<f64>,
    /// Links to related resources. Possible link 'rel' values are: 'dataset'
    /// for a URL pointing to the dataset, 'tiles' for a URL template to get
    /// the tiles; 'alternate' for a URL pointing to another representation of
    /// the TileSetMetadata (e.g a TileJSON file); 'tiling-scheme' for a
    /// definition of the [TileMatrixSet]
    links: Links,
    layers: Option<Vec<GeospatialData>>,
    /// Minimum bounding rectangle surrounding the tile matrix set, in the supported CRS
    bounding_box: Option<BoundingBox2D>,
    /// Style involving all layers used to generate the tileset
    style: Option<Style>,
    /// Location of a tile that nicely represents the tileset. Implementations
    /// may use this center value to set the default location or to present a
    /// representative tile in a user interface
    center_point: Option<TilePoint>,
    /// License applicable to the tiles
    license: Option<String>,
    /// Restrictions on the availability of the Tile Set that the user needs to
    /// be aware of before using or redistributing the Tile Set
    access_constraints: Option<AccessConstraints>,
    /// Version of the Tile Set. Changes if the data behind the tiles has been changed
    version: Option<String>,
    /// When the Tile Set was first produced
    created: Option<DateTime<Utc>>,
    /// Last Tile Set change/revision
    updated: Option<DateTime<Utc>>,
    /// Useful information to contact the authors or custodians for the Tile Set
    point_of_contact: Option<String>,
    /// Media types available for the tiles
    media_types: Option<Vec<String>>,
}

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeospatialData {
    #[serde(flatten)]
    title_description_keywords: TitleDescriptionKeywords,
    /// Unique identifier of the Layer. Implemetion of 'identifier'
    id: String,
    /// Type of data represented in the layer
    data_type: DataType,
    /// The geometry type of the features shown in this layer
    geometry_dimension: Option<GeometryDimension>,
    /// Feature type identifier. Only applicable to layers of datatype 'geometries'
    feature_type: Option<String>,
    /// Useful information to contact the authors or custodians for the layer
    /// (e.g. e-mail address, a physical address,  phone numbers, etc)
    point_of_contact: Option<String>,
    /// Organization or individual responsible for making the layer available
    publisher: Option<String>,
    /// Category where the layer can be grouped
    theme: Option<String>,
    /// Coordinate Reference System (CRS)
    crs: Option<Crs>,
    /// Epoch of the Coordinate Reference System (CRS)
    epoch: Option<f64>,
    /// Minimum scale denominator for usage of the layer
    min_scale_denominator: Option<f64>,
    /// aximum scale denominator for usage of the layer
    max_scale_denominator: Option<f64>,
    /// Minimum cell size for usage of the layer
    min_cell_size: Option<f64>,
    /// Maximum cell size for usage of the layer
    max_cell_size: Option<f64>,
    /// TileMatrix identifier associated with the minScaleDenominator
    max_tile_matrix: Option<String>,
    /// TileMatrix identifier associated with the maxScaleDenominator
    min_tile_matrix: Option<String>,
    /// Minimum bounding rectangle surrounding the layer
    bounding_box: Option<BoundingBox2D>,
    /// When the layer was first produced
    created: Option<DateTime<Utc>>,
    /// Last layer change/revision
    updated: Option<DateTime<Utc>>,
    /// Style used to generate the layer in the tileset
    style: Option<Style>,
    /// URI identifying a class of data contained in this layer (useful to
    /// determine compatibility with styles or processes)
    geo_data_classes: Option<Vec<String>>,
    /// Properties represented by the features in this layer. Can be the
    /// attributes of a feature dataset (datatype=geometries) or the rangeType
    /// of a coverage (datatype=coverage)
    properties_schema: Option<Value>,
    /// Links related to this layer. Possible link 'rel' values are:
    /// 'geodata' for a URL pointing to the collection of geospatial data.
    links: Option<Links>,
}

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TilePoint {
    coordinates: Option<Point2D>,
    // Coordinate Reference System (CRS) of the coordinates
    crs: Option<Crs>,
    /// TileMatrix identifier associated with the scaleDenominator
    tile_matrix: Option<String>,
    /// Scale denominator of the tile matrix selected
    scale_denominator: Option<f64>,
    /// Cell size of the tile matrix selected
    cell_size: Option<f64>,
}

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Style {
    #[serde(flatten)]
    title_description_keywords: TitleDescriptionKeywords,
    /// An identifier for this style. Implementation of 'identifier'
    id: String,
    /// Links to style related resources. Possible link 'rel' values are:
    /// 'style' for a URL pointing to the style description, 'styleSpec' for a
    /// URL pointing to the specification or standard used to define the style.
    links: Option<Links>,
}

/// A resource describing useful to create an array that describes the limits
/// for a tile set [TileMatrixSet] based on the OGC TileSet Metadata Standard
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TileMatrixLimits {
    tile_matrix: String,
    min_tile_row: u64,
    max_tile_row: u64,
    min_tile_col: u64,
    max_tile_col: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DataType {
    Map,
    Vector,
    Coverage,
}

#[repr(u8)]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug)]
enum GeometryDimension {
    Points = 0,
    Curves = 1,
    Surfaces = 2,
    Solids = 3,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum AccessConstraints {
    Unclassified,
    Restricted,
    Confidential,
    Secret,
    TopSecret,
}

impl Default for AccessConstraints {
    fn default() -> Self {
        AccessConstraints::Unclassified
    }
}

#[cfg(test)]
mod test {
    use super::GeometryDimension;

    #[test]
    fn geometry_dimension() {
        assert_eq!(
            serde_json::from_str::<GeometryDimension>("0").unwrap(),
            GeometryDimension::Points
        );
        assert_eq!(
            serde_json::to_value(&GeometryDimension::Points)
                .unwrap()
                .as_u64(),
            Some(0)
        );
    }
}

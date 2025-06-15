use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use utoipa::ToSchema;

use crate::common::Link;

use super::{BoundingBox2D, Point2D, TilesCrs};

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TileSets {
    pub tilesets: Vec<TileSetItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

/// A minimal tileset element for use within a list of tilesets linking to
/// full description of those tilesets.
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TileSetItem {
    /// A title for this tileset
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub title: Option<String>,
    /// Type of data represented in the tileset
    #[schema(inline = true)]
    pub data_type: DataType,
    /// Coordinate Reference System (CRS)
    pub crs: TilesCrs,
    /// Reference to a Tile Matrix Set on an offical source for Tile Matrix Sets
    /// such as the OGC NA definition server (http://www.opengis.net/def/tms/).
    /// Required if the tile matrix set is registered on an open official source.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "tileMatrixSetURI"
    )]
    #[schema(nullable = false, format = Uri)]
    pub tile_matrix_set_uri: Option<String>,
    /// Links to related resources. A 'self' link to the tileset as well as a
    /// 'http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme' link to a
    /// definition of the TileMatrixSet are required.
    pub links: Vec<Link>,
}

/// A resource describing a tileset based on the OGC TileSet Metadata Standard.
/// At least one of the 'TileMatrixSet',  or a link with 'rel' tiling-scheme"
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TileSet {
    /// A title for this tileset
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub title: Option<String>,
    /// Brief narrative description of this tile set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub description: Option<String>,
    /// Unordered list of one or more commonly used or formalized word(s) or
    /// phrase(s) used to describe a TileSet
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Type of data represented in the tileset
    #[schema(inline = true)]
    pub data_type: DataType,
    /// Reference to a Tile Matrix Set on the OGC NA definition server
    /// (<http://www.opengis.net/def/tms/>). Required if the tile matrix set is
    /// registered on the definition server.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "tileMatrixSetURI"
    )]
    #[schema(nullable = false, format = Uri)]
    pub tile_matrix_set_uri: Option<String>,
    /// Limits for the TileRow and TileCol values for each TileMatrix in the
    /// TileMatrixSet. If missing, there are no limits other that the ones
    /// imposed by the TileMatrixSet. If present the TileMatrices listed are
    /// limited and the rest not available at all
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tile_matrix_set_limits: Vec<TileMatrixLimits>,
    /// Coordinate Reference System (CRS)
    pub crs: TilesCrs,
    /// Epoch of the Coordinate Reference System (CRS)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub epoch: Option<f64>,
    /// Links to related resources. Possible link 'rel' values are: 'dataset'
    /// for a URL pointing to the dataset, 'tiles' for a URL template to get
    /// the tiles; 'alternate' for a URL pointing to another representation of
    /// the TileSetMetadata (e.g a TileJSON file); 'tiling-scheme' for a
    /// definition of the TileMatrixSet
    pub links: Vec<Link>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(min_items = 1)]
    pub layers: Vec<GeospatialData>,
    /// Minimum bounding rectangle surrounding the tile matrix set, in the supported CRS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub bounding_box: Option<BoundingBox2D>,
    /// Location of a tile that nicely represents the tileset. Implementations
    /// may use this center value to set the default location or to present a
    /// representative tile in a user interface
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub center_point: Option<TilePoint>,
    /// Style involving all layers used to generate the tileset
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub style: Option<Style>,
    /// Short reference to recognize the author or provider
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub attribution: Option<String>,
    /// License applicable to the tiles
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub license: Option<String>,
    /// Restrictions on the availability of the Tile Set that the user needs to
    /// be aware of before using or redistributing the Tile Set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub access_constraints: Option<AccessConstraints>,
    /// Version of the Tile Set. Changes if the data behind the tiles has been changed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub version: Option<String>,
    /// When the Tile Set was first produced
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub created: Option<DateTime<Utc>>,
    /// Last Tile Set change/revision
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub updated: Option<DateTime<Utc>>,
    /// Useful information to contact the authors or custodians for the Tile Set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub point_of_contact: Option<String>,
    /// Media types available for the tiles
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media_types: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeospatialData {
    /// Unique identifier of the Layer.
    pub id: String,
    /// Title for this layer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub title: Option<String>,
    /// Brief narrative description of this layer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub description: Option<String>,
    /// Unordered list of one or more commonly used or formalized word(s) or
    /// phrase(s) used to describe this layer
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Type of data represented in the layer
    #[schema(inline = true)]
    pub data_type: DataType,
    /// The geometry dimension of the features shown in this layer (0: points,
    /// 1: curves, 2: surfaces, 3: solids), unspecified: mixed or unknown
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub geometry_dimension: Option<GeometryDimension>,
    /// Feature type identifier. Only applicable to layers of datatype 'geometries'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub feature_type: Option<String>,
    /// Short reference to recognize the author or provider
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub attribution: Option<String>,
    /// License applicable to the tiles
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub license: Option<String>,
    /// Useful information to contact the authors or custodians for the layer
    /// (e.g. e-mail address, a physical address,  phone numbers, etc)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub point_of_contact: Option<String>,
    /// Organization or individual responsible for making the layer available
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub publisher: Option<String>,
    /// Category where the layer can be grouped
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub theme: Option<String>,
    /// Coordinate Reference System (CRS)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crs: Option<TilesCrs>,
    /// Epoch of the Coordinate Reference System (CRS)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub epoch: Option<f64>,
    /// Minimum scale denominator for usage of the layer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub min_scale_denominator: Option<f64>,
    /// aximum scale denominator for usage of the layer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub max_scale_denominator: Option<f64>,
    /// Minimum cell size for usage of the layer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub min_cell_size: Option<f64>,
    /// Maximum cell size for usage of the layer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub max_cell_size: Option<f64>,
    /// TileMatrix identifier associated with the minScaleDenominator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub max_tile_matrix: Option<String>,
    /// TileMatrix identifier associated with the maxScaleDenominator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub min_tile_matrix: Option<String>,
    /// Minimum bounding rectangle surrounding the layer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub bounding_box: Option<BoundingBox2D>,
    /// When the layer was first produced
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub created: Option<DateTime<Utc>>,
    /// Last layer change/revision
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub updated: Option<DateTime<Utc>>,
    /// Style used to generate the layer in the tileset
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub style: Option<Style>,
    /// URI identifying a class of data contained in this layer (useful to
    /// determine compatibility with styles or processes)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub geo_data_classes: Vec<String>,
    /// Properties represented by the features in this layer. Can be the
    /// attributes of a feature dataset (datatype=geometries) or the rangeType
    /// of a coverage (datatype=coverage)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub properties_schema: Option<Value>,
    /// Links related to this layer. Possible link 'rel' values are:
    /// 'geodata' for a URL pointing to the collection of geospatial data.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TilePoint {
    #[schema(value_type = Vec<f64>, min_items = 2, max_items = 2, inline = true)]
    pub coordinates: Point2D,
    // Coordinate Reference System (CRS) of the coordinates
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub crs: Option<TilesCrs>,
    /// TileMatrix identifier associated with the scaleDenominator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub tile_matrix: Option<String>,
    /// Scale denominator of the tile matrix selected
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub scale_denominator: Option<f64>,
    /// Cell size of the tile matrix selected
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub cell_size: Option<f64>,
}

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    /// An identifier for this style. Implementation of 'identifier'
    pub id: String,
    /// A title for this style
    #[schema(nullable = false)]
    pub title: Option<String>,
    /// Brief narrative description of this style
    #[schema(nullable = false)]
    pub description: Option<String>,
    /// Keywords about this style
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Links to style related resources. Possible link 'rel' values are:
    /// 'style' for a URL pointing to the style description, 'styleSpec' for a
    /// URL pointing to the specification or standard used to define the style.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(min_items = 1)]
    pub links: Vec<Link>,
}

/// A resource describing useful to create an array that describes the limits
/// for a tile set [super::TileMatrixSet] based on the OGC TileSet Metadata Standard
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrixLimits {
    pub tile_matrix: String,
    pub min_tile_row: u64,
    pub max_tile_row: u64,
    pub min_tile_col: u64,
    pub max_tile_col: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Map,
    Vector,
    Coverage,
    #[serde(untagged)]
    String(String),
}

#[repr(u8)]
#[derive(Serialize_repr, Deserialize_repr, ToSchema, PartialEq, Eq, Debug)]
pub enum GeometryDimension {
    Points = 0,
    Curves = 1,
    Surfaces = 2,
    Solids = 3,
}

#[derive(Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
#[schema(default = "unclassified")]
pub enum AccessConstraints {
    #[default]
    Unclassified,
    Restricted,
    Confidential,
    Secret,
    TopSecret,
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

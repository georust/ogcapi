use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    pub r#type: CoverageType,
    pub domain: Domain,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub parameters: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub ranges: Map<String, Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    pub r#type: String,
    pub domain_type: Option<DomainType>,
    pub axes: Map<String, Value>,
    pub referencing: Vec<Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum CoverageType {
    Domain,
    NdArray,
    TiledNdArray,
    Coverage,
    CoverageCollection,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum DomainType {
    Grid,
    VerticalProfile,
    PointSeries,
    Point,
    MultiPointSeries,
    MultiPoint,
    PolygonSeries,
    Polygon,
    MultiPolygonSeries,
    MultiPolygon,
    Trajectory,
    Section,
}

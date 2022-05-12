use serde::{Deserialize, Serialize};

use crate::common::Link;

use super::QueryType;

/// Detailed information relevant to individual query types
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DataQueries {
    pub position: Option<PositionLink>,
    // pub radius: Option<RadiusLink>,
    // pub area: Option<AreaLink>,
    // pub cube: Option<CubeLink>,
    // pub trajectory: Option<TrajectoryLink>,
    // pub corridor: Option<CorridorLink>,
    // pub location: Option<LocationLink>,
    // pub item: Option<ItemLink>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PositionLink {
    #[serde(flatten)]
    pub link: Link,
    pub variables: PositionDataQuery,
}

/// Property to contain any extra metadata information that is specific
/// to an individual data queries
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PositionDataQuery {
    pub title: Option<String>,
    pub description: Option<String>,
    pub query_type: QueryType,
    pub output_formats: Vec<String>,
    pub default_output_format: Option<String>,
    pub crs_details: Vec<CrsObject>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CrsObject {
    /// name of the coordinate reference system, used as the value in the crs
    /// query parameter to define the required output CRS
    pub crs: String,
    /// Well Known text description of the coordinate reference system
    pub wkt: String,
}

use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use utoipa::{IntoParams, ToSchema};

use crate::common::{Crs, Datetime};

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum QueryType {
    Position,
    Radius,
    Area,
    Cube,
    Trajectory,
    Corridor,
    Items,
    Locations,
    Instances,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, IntoParams, Default, Debug)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Query {
    /// Well Known Text (WKT) of representation geometry. The representation
    /// type will depend on the [QueryType] of the API.
    #[serde(alias = "bbox")]
    pub coords: String,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[param(value_type = String)]
    pub datetime: Option<Datetime>,
    pub parameter_name: Option<String>,
    #[serde(default)]
    #[param(value_type = String)]
    pub crs: Option<Crs>,
    pub f: Option<String>,
    pub z: Option<Vec<String>>,
    pub within: Option<String>,
    pub within_units: Option<String>,
    pub resolution_x: Option<usize>,
    pub resolution_z: Option<usize>,
    pub corridor_height: Option<String>,
    pub height_units: Option<String>,
    pub corridor_width: Option<String>,
    pub width_units: Option<String>,
}

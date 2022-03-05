use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::common::{core::Datetime, crs::Crs};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct Provider {
    /// Name of organization providing the service
    name: Option<String>,
    /// Link to service providers website
    url: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    email: Option<String>,
    phone: Option<String>,
    fax: Option<String>,
    hours: Option<String>,
    insructions: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    city: Option<String>,
    stateorprovince: Option<String>,
    country: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

#[serde_as]
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct EdrQuery {
    /// Well Known Text (WKT) of representation geometry. The representation
    /// type will depend on the [QueryType] of the API.
    #[serde(alias = "bbox")]
    pub coords: String,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub datetime: Option<Datetime>,
    pub parameter_name: Option<String>,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub crs: Crs,
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

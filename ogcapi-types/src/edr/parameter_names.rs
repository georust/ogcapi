use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::common::Extent;

use super::{ObservedPropertyCollection, Units};

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParameterNames {
    #[serde(default)]
    #[schema(inline)]
    pub r#type: Type,
    /// Unique ID of the parameter, this is the value used for querying the data
    pub id: Option<String>,
    pub description: Option<Value>,
    pub label: Option<String>,
    #[serde(rename = "data-type")]
    pub data_type: Option<DataType>,
    pub unit: Option<Units>,
    pub observed_property: ObservedPropertyCollection,
    pub category_encoding: Option<Value>,
    pub extent: Option<Extent>,
    pub measurement_type: Option<MeasurementType>,
}

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, PartialEq, Eq, Clone)]
pub enum Type {
    #[default]
    Parameter,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Integer,
    Float,
    String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
pub struct MeasurementType {
    /// Approach to calculating the data values
    pub method: String,
    /// The time duration that the value was calculated for as an RFC3339
    /// duration value. If the method value is instantaneous this value is
    /// not required.
    pub duration: Option<String>,
}

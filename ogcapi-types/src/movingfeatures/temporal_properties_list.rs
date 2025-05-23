use serde::{Deserialize, Serialize};

use crate::common::Links;

use super::{temporal_properties::TemporalProperties, temporal_property::TemporalProperty};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalPropertiesList {
    pub temporal_properties: TemporalPropertiesListValue,
    pub links: Option<Links>,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum TemporalPropertiesListValue {
    MFJsonTemporalProperties(Vec<TemporalProperties>),
    TemporalProperty(Vec<TemporalProperty>),
}

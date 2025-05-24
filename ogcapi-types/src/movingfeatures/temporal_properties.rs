use serde::{Deserialize, Serialize};

use crate::common::Links;

use super::{mfjson_temporal_properties::MFJsonTemporalProperties, temporal_property::TemporalProperty};

/// A TemporalProperties object consists of the set of [TemporalProperty] or a set of [MFJsonTemporalProperties].
///
/// See [8.8 TemporalProperties](https://docs.ogc.org/is/22-003r3/22-003r3.html#resource-temporalProperties-section)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalProperties {
    pub temporal_properties: TemporalPropertiesValue,
    pub links: Option<Links>,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum TemporalPropertiesValue {
    /// [MFJsonTemporalProperties] allows to represent multiple property values all measured at the same points in time.
    MFJsonTemporalProperties(Vec<MFJsonTemporalProperties>),
    /// [TemporalProperty] allows to represent a property value at independent points in time.
    TemporalProperty(Vec<TemporalProperty>),
}


use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Link;

#[cfg(feature = "movingfeatures")]
use crate::common::Bbox;

use super::Feature;

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[default]
    FeatureCollection,
}

/// A set of Features from a dataset
#[derive(Serialize, Deserialize, ToSchema, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollection {
    #[serde(default)]
    #[schema(inline = true)]
    pub r#type: Type,
    pub features: Vec<Feature>,
    #[serde(default)]
    pub links: Vec<Link>,
    /// This property indicates the time and date when the response was generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    /// The number of features of the feature type that match the selection
    /// parameters like `bbox`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<u64>,
    /// The number of features in the feature collection.
    ///
    /// A server may omit this information in a response, if the information
    /// about the number of features is not known or difficult to compute.
    ///
    /// If the value is provided, the value shall be identical to the number
    /// of items in the "features" array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<u64>,
    #[cfg(feature = "movingfeatures")]
    #[serde(default)]
    pub crs: crate::movingfeatures::crs::Crs,
    #[cfg(feature = "movingfeatures")]
    #[serde(default)]
    pub trs: crate::movingfeatures::trs::Trs,
    #[cfg(feature = "movingfeatures")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Bbox>,
}

impl FeatureCollection {
    pub fn new(features: Vec<Feature>) -> Self {
        let number_returned = features.len();
        FeatureCollection {
            features,
            time_stamp: Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
            number_returned: Some(number_returned as u64),
            ..Default::default()
        }
    }
}

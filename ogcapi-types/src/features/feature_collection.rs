use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::common::Links;

#[cfg(feature = "movingfeatures")]
use crate::common::Bbox;

use super::Feature;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub enum Type {
    #[default]
    FeatureCollection,
}

/// A set of Features from a dataset
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollection {
    #[serde(default)]
    pub r#type: Type,
    pub features: Vec<Feature>,
    #[serde(default)]
    pub links: Links,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<u64>,
    #[cfg(feature = "movingfeatures")]
    pub crs: Option<crate::movingfeatures::crs::Crs>,
    #[cfg(feature = "movingfeatures")]
    pub trs: Option<crate::movingfeatures::trs::Trs>,
    #[cfg(feature = "movingfeatures")]
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

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Link;

use super::Feature;

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[default]
    FeatureCollection,
}

/// A set of Features from a dataset
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, ToSchema, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollection {
    #[serde(default)]
    #[schema(inline = true)]
    pub r#type: Type,
    pub features: Vec<Feature>,
    #[serde(default)]
    pub links: Vec<Link>,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<u64>,
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

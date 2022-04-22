mod query;

pub use query::Query;

use std::collections::HashMap;

pub use geojson::{Bbox, Geometry};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::Links;

/// A set of Features from a dataset
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollection {
    #[serde(default = "feature_collection")]
    pub r#type: String,
    pub features: Vec<Feature>,
    #[serde(default)]
    pub links: Links,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<usize>,
}

fn feature_collection() -> String {
    "FeatureCollection".to_string()
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Feature {
    pub id: Option<String>,
    pub collection: Option<String>,
    #[serde(default = "feature")]
    pub r#type: String,
    #[serialize_always]
    pub properties: Option<HashMap<String, Value>>,
    pub geometry: Geometry,
    #[serde(default)]
    pub links: Links,
    /// The STAC version the Item implements.
    #[cfg(feature = "stac")]
    #[serde(rename = "stac_version")]
    pub stac_version: String,
    /// A list of extensions the Item implements.
    #[serde(default)]
    #[cfg(feature = "stac")]
    #[serde(rename = "stac_extensions")]
    pub stac_extensions: Vec<String>,
    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    #[serde(default)]
    #[cfg(feature = "stac")]
    pub assets: HashMap<String, crate::stac::Asset>,
    /// Bounding Box of the asset represented by this Item, formatted according to RFC 7946, section 5.
    #[cfg(feature = "stac")]
    pub bbox: Option<Bbox>,
}

fn feature() -> String {
    "Feature".to_string()
}

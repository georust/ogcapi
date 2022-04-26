#[cfg(feature = "stac")]
use std::collections::HashMap;

#[cfg(feature = "stac")]
use crate::common::Bbox;
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::common::Links;

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Feature {
    pub id: Option<String>,
    pub collection: Option<String>,
    #[serde(default = "feature")]
    pub r#type: String,
    #[serialize_always]
    pub properties: Option<Map<String, Value>>,
    pub geometry: Geometry,
    #[serde(default)]
    pub links: Links,
    /// The STAC version the Item implements.
    #[cfg(feature = "stac")]
    #[serde(default = "crate::stac::stac_version", rename = "stac_version")]
    pub stac_version: String,
    /// A list of extensions the Item implements.
    #[cfg(feature = "stac")]
    #[serde(default, rename = "stac_extensions")]
    pub stac_extensions: Vec<String>,
    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    #[cfg(feature = "stac")]
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub assets: HashMap<String, crate::stac::Asset>,
    /// Bounding Box of the asset represented by this Item, formatted according to RFC 7946, section 5.
    #[cfg(feature = "stac")]
    pub bbox: Option<Bbox>,
}

fn feature() -> String {
    "Feature".to_string()
}

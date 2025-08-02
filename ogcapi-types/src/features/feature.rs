#[cfg(feature = "stac")]
use std::collections::HashMap;
use std::fmt::Display;

#[cfg(feature = "stac")]
use crate::common::Bbox;
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::common::Links;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub enum Type {
    #[default]
    Feature,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum FeatureId {
    String(String),
    Integer(isize),
}

impl Display for FeatureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureId::String(s) => f.write_str(s),
            FeatureId::Integer(i) => f.write_fmt(format_args!("{i}")),
        }
    }
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Feature {
    #[serde(default)]
    pub id: Option<FeatureId>,
    pub collection: Option<String>,
    #[serde(default)]
    pub r#type: Type,
    #[serialize_always]
    pub properties: Option<Map<String, Value>>,
    pub geometry: Geometry,
    #[serde(default)]
    pub links: Links,
    /// The STAC version the Item implements.
    #[cfg(feature = "stac")]
    #[serde(default = "crate::stac::stac_version")]
    pub stac_version: String,
    /// A list of extensions the Item implements.
    #[cfg(feature = "stac")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stac_extensions: Vec<String>,
    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    #[cfg(feature = "stac")]
    #[serde(default)]
    pub assets: HashMap<String, crate::stac::Asset>,
    /// Bounding Box of the asset represented by this Item, formatted according to RFC 7946, section 5.
    #[cfg(feature = "stac")]
    #[serde(default)]
    pub bbox: Option<Bbox>,
}

impl Feature {
    pub fn append_properties(&mut self, mut other: Map<String, Value>) {
        if let Some(properties) = self.properties.as_mut() {
            properties.append(&mut other);
        } else {
            self.properties = Some(other);
        }
    }
}

#[cfg(feature = "stac")]
use std::collections::HashMap;

#[cfg(feature = "stac")]
use crate::common::Bbox;
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::{ToSchema, openapi::Schema};

use crate::common::Link;

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[default]
    Feature,
}

/// Geometry schema.
pub fn geometry() -> Schema {
    serde_json::from_str(include_str!("../../assets/schema/Geometry.json")).unwrap()
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone, PartialEq)]
pub struct Feature {
    pub id: Option<String>,
    pub collection: Option<String>,
    #[serde(default)]
    #[schema(inline = true)]
    pub r#type: Type,
    #[serialize_always]
    pub properties: Option<Map<String, Value>>,
    #[schema(schema_with = geometry)]
    pub geometry: Geometry,
    /// Bounding Box of the asset represented by this Item, formatted according to RFC 7946, section 5.
    #[cfg(feature = "stac")]
    pub bbox: Option<Bbox>,
    #[serde(default)]
    pub links: Vec<Link>,
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

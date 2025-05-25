#[cfg(feature = "stac")]
use std::collections::HashMap;
use std::fmt::Display;

#[cfg(feature = "stac")]
use crate::common::Bbox;

#[cfg(feature = "movingfeatures")]
use crate::movingfeatures::{
    crs::Crs, temporal_geometry::TemporalGeometry, temporal_properties::TemporalProperties,
    trs::Trs,
};

#[cfg(feature = "movingfeatures")]
use chrono::{DateTime, Utc};

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

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
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

/// Geometry schema.
pub fn geometry() -> Schema {
    serde_json::from_str(include_str!("../../assets/schema/Geometry.json")).unwrap()
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[serde_with::skip_serializing_none]
<<<<<<< HEAD
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone, PartialEq)]
=======
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename = "camelCase")]
>>>>>>> ba7e7ad (Integrate with moving features with collection, feature collection and features)
pub struct Feature {
    #[serde(default)]
    pub id: Option<FeatureId>,
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
    #[cfg(feature = "movingfeatures")]
    #[serde(serialize_with = "crate::common::serialize_interval")]
    /// Life span information for the moving feature.
    /// See [MF-Json 7.2.3 LifeSpan](https://docs.ogc.org/is/19-045r3/19-045r3.html#time)
    pub time: Vec<Vec<Option<DateTime<Utc>>>>,
    #[cfg(feature = "movingfeatures")]
    // TODO should this be #[serde(default)] instead of option?
    pub crs: Option<Crs>,
    #[cfg(feature = "movingfeatures")]
    // TODO should this be #[serde(default)] instead of option?
    pub trs: Option<Trs>,
    #[cfg(feature = "movingfeatures")]
    pub temporal_geometry: Option<TemporalGeometry>,
    #[cfg(feature = "movingfeatures")]
    pub temporal_properties: Option<TemporalProperties>,
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

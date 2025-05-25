#[cfg(feature = "stac")]
use std::collections::HashMap;

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

use crate::common::Links;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub enum Type {
    #[default]
    Feature,
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename = "camelCase")]
pub struct Feature {
    pub id: Option<String>,
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
    pub bbox: Option<Bbox>,
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

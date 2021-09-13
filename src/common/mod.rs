pub mod collections;
pub mod core;
mod crs;

pub use self::crs::{Crs, OGC_CRS84h};

use serde::{Deserialize, Serialize};

/// Content Type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ContentType {
    #[serde(rename = "application/json")]
    JSON,
    #[serde(rename = "application/geo+json")]
    GeoJSON,
    #[serde(rename = "application/vnd.oai.openapi+json;version=3.0")]
    OpenAPI,
    #[serde(rename = "application/vnd.mapbox.style+json")]
    MapboxStyle,
    #[serde(rename = "application/vnd.ogc.sld+xml;version=1.0")]
    SLD,
    #[serde(rename = "text/html")]
    HTML,
}

mod bbox;
pub mod collections;
pub mod core;
mod crs;
mod datetime;

pub use self::bbox::Bbox;
pub use self::crs::{OGC_CRS84h, CRS, OGC_CRS84};
pub use self::datetime::Datetime;

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tide::http::Mime;

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
}

impl Into<Mime> for ContentType {
    fn into(self) -> Mime {
        match self {
            ContentType::JSON => Mime::from_str("application/json").unwrap(),
            ContentType::GeoJSON => Mime::from_str("application/geo+json").unwrap(),
            ContentType::OpenAPI => {
                Mime::from_str("application/vnd.oai.openapi+json;version=3.0").unwrap()
            }
            ContentType::MapboxStyle => {
                Mime::from_str("application/vnd.mapbox.style+json").unwrap()
            }
            ContentType::SLD => Mime::from_str("application/vnd.ogc.sld+xml;version=1.0").unwrap(),
        }
    }
}

#[test]
fn content_type() {
    let _mime: Mime = ContentType::OpenAPI.into();
}

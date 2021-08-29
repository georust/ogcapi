mod bbox;
mod crs;
mod datetime;
mod exception;
mod link;

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tide::http::Mime;

pub use self::bbox::BBOX;
pub use self::crs::{OGC_CRS84h, CRS, OGC_CRS84};
pub use self::datetime::Datetime;
pub use self::exception::Exception;
pub use self::link::*;

/// The Landing page is the entry point of a OGC API
///
/// The Landing page provides links to:
///
/// * the API definition (link relations service-desc and service-doc),
///
/// * the Conformance declaration (path /conformance, link relation conformance), and
///
/// * the Collections (path /collections, link relation data).
#[derive(Serialize, Deserialize, Default)]
pub struct LandingPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>, // OAF Core 1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>, // OAF Core 1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    pub links: Links, // OAF Core 1.0
}

/// The Conformance declaration states the conformance classes from standards or community
/// specifications, identified by a URI, that the API conforms to. Clients can but are not
/// required to use this information. Accessing the Conformance declaration using HTTP GET
/// returns the list of URIs of conformance classes implemented by the server.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conformance {
    pub conforms_to: Vec<String>,
}
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

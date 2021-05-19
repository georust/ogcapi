mod bbox;
mod crs;
mod datetime;
mod link;

use serde::{Deserialize, Serialize};

pub use self::bbox::BBOX;
pub use self::crs::CRS;
pub use self::datetime::Datetime;
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
    pub links: Vec<Link>, // OAF Core 1.0
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

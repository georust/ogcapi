use serde::{Deserialize, Serialize};

/// Media Type definitions used in the OGC API standards
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MediaType {
    /// Media Type for `text/html`
    #[serde(rename = "text/html")]
    HTML,
    /// Media Type for `application/json`
    #[serde(rename = "application/json")]
    JSON,
    /// Media Type for `application/geo+json`
    #[serde(rename = "application/geo+json")]
    GeoJSON,
    /// Media Type for `application/vnd.oai.openapi;version=3.0`
    #[serde(rename = "application/vnd.oai.openapi;version=3.0")]
    OpenAPI,
    /// Media Type for `application/vnd.oai.openapi+json;version=3.0`
    #[serde(rename = "application/vnd.oai.openapi+json;version=3.0")]
    OpenAPIJson,
    /// Media Type for `application/vnd.oai.openapi+yaml;version=3.0`
    #[serde(rename = "application/vnd.oai.openapi+yaml;version=3.0")]
    OpenAPIYaml,
    /// Media Type for `application/vnd.mapbox.style+json`
    #[serde(rename = "application/vnd.mapbox.style+json")]
    MapboxStyle,
    /// Media Type for `application/vnd.ogc.sld+xml;version=1.0`
    #[serde(rename = "application/vnd.ogc.sld+xml;version=1.0")]
    SLD,
    /// Media Type for `application/problem+json`
    #[serde(rename = "application/problem+json")]
    ProblemJSON,
    /// Media Type for `image/png`
    #[serde(rename = "image/png")]
    PNG,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_value(self).unwrap().as_str().unwrap()
        )
    }
}

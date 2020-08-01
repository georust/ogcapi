use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tide::http::Mime;

/// Hyperlink to enable Hypermedia Access
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Link {
    pub href: String,
    pub rel: LinkRelation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ContentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
}

/// Link Relations
///
/// [IANA Link Relations Registry](https://www.iana.org/assignments/link-relations/link-relations.xhtml)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum LinkRelation {
    Alternate,
    Collection,
    Current,
    Conformance,
    Data,
    Decribedby,
    Exceptions,
    Execute,
    First,
    Item,
    Items,
    Last,
    License,
    Next,
    Previous,
    ProcessDesc,
    Processes,
    Results,
    #[serde(rename = "self")]
    Selfie,
    ServiceDesc,
    ServiceDoc,
    Start,
    Status,
    Tiles,
    Up,
}

impl Default for LinkRelation {
    fn default() -> Self {
        LinkRelation::Selfie
    }
}

/// Link Content Type
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContentType {
    #[serde(rename = "application/json")]
    JSON,
    #[serde(rename = "application/geo+json")]
    GEOJSON,
    #[serde(rename = "application/vnd.oai.openapi+json;version=3.0")]
    OPENAPI,
}

impl ContentType {
    fn as_str(&self) -> &'static str {
        match self {
            ContentType::JSON => "application/json",
            ContentType::GEOJSON => "application/geo+json",
            ContentType::OPENAPI => "application/vnd.oai.openapi+json;version=3.0",
        }
    }
}

impl Into<Mime> for ContentType {
    fn into(self) -> Mime {
        Mime::from_str(&self.as_str()).unwrap()
    }
}

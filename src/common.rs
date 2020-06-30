use serde::{Deserialize, Serialize};

/// Hyperlink to enable Hypermedia Access
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
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
    First,
    Item,
    Items,
    Last,
    License,
    Next,
    Previous,
    #[serde(rename = "self")]
    Selfie,
    ServiceDesc,
    ServiceDoc,
    Start,
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
    Json,
    #[serde(rename = "application/geo+json")]
    GeoJson,
    #[serde(rename = "application/vnd.oai.openapi+json;version=3.0")]
    OpenAPI,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LandingPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub links: Vec<Link>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize)]
pub struct Conformance {
    pub conforms_to: Vec<String>,
}

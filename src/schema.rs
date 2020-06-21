use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LandingPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct Conformance {
    #[serde(rename(serialize = "conformsTo", deserialize = "conformsTo"))]
    pub conforms_to: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Collections {
    links: Vec<Link>,
    collections: Vec<Collection>,
}

#[derive(Serialize, Deserialize)]
struct Collection {
    id: String,
    title: Option<String>,
    description: Option<String>,
    links: Vec<Link>,
    extent: Option<Extent>,
    #[serde(rename(serialize = "itemType", deserialize = "item_type"))]
    item_type: Option<String>,
    crs: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct Extent {
    spatial: Option<SpatialExtent>,
    temporal: Option<TemporalExtent>,
}

#[derive(Serialize, Deserialize)]
struct SpatialExtent {
    bbox: Option<Vec<Vec<f64>>>,
    crs: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TemporalExtent {
    interval: Option<Vec<Vec<String>>>,
    trs: Option<String>,
}

// #[derive(Serialize, Deserialize)]
// pub struct ServerError {
//     pub code: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub description: Option<String>
// }

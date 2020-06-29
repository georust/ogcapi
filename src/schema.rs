use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
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
    pub length: Option<usize>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct LandingPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize)]
pub struct Conformance {
    #[serde(rename(serialize = "conformsTo", deserialize = "conformsTo"))]
    pub conforms_to: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Collections {
    pub links: Vec<Link>,
    pub collections: Vec<Collection>,
}

#[derive(Serialize, Deserialize, Default, Debug, sqlx::FromRow)]
pub struct Collection {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub links: Vec<Json<Link>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extent: Option<Json<Extent>>,
    #[serde(rename(serialize = "itemType", deserialize = "item_type"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Extent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial: Option<Json<SpatialExtent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporal: Option<Json<TemporalExtent>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpatialExtent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<Vec<f64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemporalExtent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trs: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct FeatureCollection {
    pub r#type: String,
    pub features: Vec<Feature>,
    pub links: Option<Vec<Link>>,
    #[serde(rename(serialize = "timeStamp", deserialize = "time_stamp"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    #[serde(rename(serialize = "numberMatched", deserialize = "number_matched"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<u64>,
    #[serde(rename(serialize = "numberReturned", deserialize = "number_returned"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<usize>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Feature {
    pub r#type: String,
    pub id: Option<String>,
    pub properties: Value,
    pub geometry: Value,
    pub links: Option<Json<Vec<Link>>>,
}

#[derive(Serialize, Deserialize)]
pub struct Exception {
    pub code: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Query {
    #[serde(default)]
    pub limit: Option<isize>,
    #[serde(default)]
    pub offset: Option<isize>,
    pub bbox: Option<String>,
    pub datetime: Option<String>,
}

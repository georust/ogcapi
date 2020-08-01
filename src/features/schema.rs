use crate::common::Link;
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;

/// A set of Features from a dataset
#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Default)]
pub struct FeatureCollection {
    pub r#type: String,
    pub features: Vec<Feature>,
    pub links: Option<Vec<Link>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<usize>,
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Feature {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub properties: Value,
    pub geometry: Json<Geometry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Json<Vec<Link>>>,
}

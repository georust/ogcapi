mod query;

pub use query::Query;

use std::collections::HashMap;

use geojson::{Bbox, Geometry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;

use crate::common::Links;

/// A set of Features from a dataset
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollection {
    pub r#type: String,
    pub features: Vec<Feature>,
    pub links: Option<Links>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<usize>,
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, PartialEq)]
pub struct Feature {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    #[serde(rename = "type")]
    pub feature_type: Json<FeatureType>,
    pub properties: Option<Value>,
    pub geometry: Json<Geometry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Json<Links>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Json<Assets>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Json<Bbox>>,
}

#[derive(sqlx::Type, Deserialize, Serialize, Debug, PartialEq)]
pub enum FeatureType {
    Feature,
    Unknown,
}

/// Dictionary of asset objects that can be downloaded, each with a unique key.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Assets {
    #[serde(flatten)]
    inner: HashMap<String, Asset>,
}

/// An asset is an object that contains a link to data associated
/// with the Item that can be downloaded or streamed. It is allowed
/// to add additional fields.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Asset {
    href: String,
    title: String,
    description: String,
    #[serde(rename = "type")]
    content_type: String, // TODO: use content type
    roles: Vec<AssetRole>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum AssetRole {
    Thumbnail,
    Overview,
    Data,
    Metadata,
}

mod query;

pub use query::Query;

use std::collections::HashMap;

pub use geojson::{Bbox, Geometry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;

use crate::common::Links;

/// A set of Features from a dataset
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollection {
    pub r#type: String,
    pub features: Vec<Feature>,
    #[serialize_always]
    pub links: Option<Links>,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<usize>,
}

/// Abstraction of real world phenomena (ISO 19101-1:2014)
#[serde_with::skip_serializing_none]
#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, PartialEq)]
pub struct Feature {
    pub id: Option<i64>,
    pub collection: Option<String>,
    #[serde(rename = "type")]
    pub feature_type: Json<FeatureType>,
    #[serialize_always]
    pub properties: Option<Json<HashMap<String, Value>>>,
    pub geometry: Json<Geometry>,
    pub links: Option<Json<Links>>,
    pub stac_version: Option<String>,
    pub stac_extensions: Option<Vec<String>>,
    pub assets: Option<Json<Assets>>,
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

mod routes;

pub use routes::*;

use crate::common::Link;
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

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
    pub id: Option<Uuid>,
    pub properties: Value,
    pub geometry: Json<Geometry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Json<Vec<Link>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<Json<AssetObject>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
}

/// An asset is an object that contains a link to data associated
/// with the Item that can be downloaded or streamed. It is allowed
/// to add additional fields.
#[derive(Serialize, Deserialize)]
pub struct AssetObject {
    href: String,
    title: String,
    description: String,
    r#type: String,
    roles: Vec<AssetRoleType>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AssetRoleType {
    Thumbnail,
    Overview,
    Data,
    Metadata,
}

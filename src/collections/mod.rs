mod routes;

pub use self::routes::*;

use crate::common::{Datetime, Link, BBOX, CRS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
use std::collections::HashMap;

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Default)]
pub struct Collections {
    pub links: Option<Vec<Link>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<usize>,
    pub collections: Vec<Collection>,
    pub crs: Option<Vec<String>>,
}

/// A body of resources that belong or are used together. An aggregate, set, or group of related resources.
#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Default, Debug, sqlx::FromRow)]
pub struct Collection {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub attribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extent: Option<Json<Extent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<Json<ItemType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_crs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_crs_coordinate_epoch: Option<f32>,
    pub links: Json<Vec<Link>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<Json<Vec<Provider>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summaries: Option<Json<Summaries>>,
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
    pub bbox: Option<Vec<BBOX>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<CRS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemporalExtent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<Vec<Datetime>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trs: Option<String>,
}

#[derive(sqlx::Type, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Feature,
    Unknown,
}

/// A provider is any of the organizations that captures or processes the content
/// of the collection and therefore influences the data offered by this collection.
#[derive(Serialize, Deserialize, Debug)]
pub struct Provider {
    name: String,
    description: Option<String>,
    roles: Option<ProviderRole>,
    url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ProviderRole {
    Licensor,
    Producer,
    Processor,
    Host,
}

/// Dictionary of asset objects that can be downloaded, each with a unique key.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Summaries {
    #[serde(flatten)]
    inner: HashMap<String, Value>,
}
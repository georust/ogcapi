use super::CRS;
use crate::common::link::Link;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Serialize, Default)]
pub struct Collections {
    pub links: Vec<Link>,
    pub time_stamp: Option<String>,
    pub number_matched: Option<usize>,
    pub number_returned: Option<usize>,
    pub crs: Vec<String>,
    pub collections: Vec<Collection>,
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
    pub links: Vec<Json<Link>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extent: Option<Json<Extent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_crs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_crs_coordinate_epoch: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<Json<Provider>>>,
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
    pub crs: Option<CRS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemporalExtent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trs: Option<String>,
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

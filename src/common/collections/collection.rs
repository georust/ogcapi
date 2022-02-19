use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};

use crate::common::core::{Bbox, Links};
use crate::common::crs::Crs;

/// A body of resources that belong or are used together. An aggregate, set, or group of related resources.
#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub attribution: Option<String>,
    pub extent: Option<Extent>,
    pub item_type: Option<ItemType>,
    #[serde(default)]
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    pub crs: Option<Vec<Crs>>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub storage_crs: Option<Crs>,
    pub storage_crs_coordinate_epoch: Option<f32>,
    pub links: Links,
    pub stac_version: Option<String>,
    pub stac_extensions: Option<Vec<String>>,
    pub licence: Option<String>,
    pub providers: Option<Vec<Provider>>,
    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    pub summaries: Option<HashMap<String, Value>>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct Extent {
    pub spatial: Option<SpatialExtent>,
    pub temporal: Option<TemporalExtent>,
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct SpatialExtent {
    pub bbox: Option<Vec<Bbox>>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub crs: Option<Crs>,
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct TemporalExtent {
    #[serde_as(as = "Option<Vec<Vec<Option<DisplayFromStr>>>>")]
    pub interval: Option<Vec<Vec<Option<DateTime<Utc>>>>>,
    pub trs: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Feature,
    Unknown,
}

/// A provider is any of the organizations that captures or processes the content
/// of the collection and therefore influences the data offered by this collection.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct Provider {
    name: String,
    description: Option<String>,
    roles: Option<ProviderRole>,
    url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProviderRole {
    Licensor,
    Producer,
    Processor,
    Host,
}

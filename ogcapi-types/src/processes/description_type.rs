use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::common::Link;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DescriptionType {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<Metadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Metadata {
    LinkedMetadata(LinkedMetadata),
    ObjectMetadata(ObjectMetadata),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkedMetadata {
    #[serde(flatten)]
    pub link: Link,
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectMetadata {
    pub role: Option<String>,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub value: Option<Map<String, Value>>,
}

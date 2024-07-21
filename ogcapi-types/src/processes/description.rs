use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::common::Link;

/// Basic description type
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

/// Process input description
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InputDescription {
    #[serde(flatten)]
    pub description_type: DescriptionType,
    pub value_passing: Vec<ValuePassing>,
    #[serde(default = "min_occurs")]
    pub min_occurs: u64,
    #[serde(default)]
    pub max_occurs: MaxOccurs,
    pub schema: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum ValuePassing {
    ByValue,
    ByReference,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MaxOccurs {
    Integer(u64),
    Unbounded(String),
}

impl Default for MaxOccurs {
    fn default() -> Self {
        Self::Integer(1)
    }
}
fn min_occurs() -> u64 {
    1
}

/// Process output description
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutputDescription {
    #[serde(flatten)]
    pub description_type: DescriptionType,
    pub schema: Value,
}

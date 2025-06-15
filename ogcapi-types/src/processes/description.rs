use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// Basic description type
#[derive(Serialize, Deserialize, ToSchema, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DescriptionType {
    #[schema(nullable = false)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[schema(nullable = false)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<Metadata>,
    #[schema(nullable = false)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<AdditionalParameters>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct Metadata {
    #[schema(nullable = false)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[schema(nullable = false)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[schema(nullable = false)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AdditionalParameters {
    #[serde(default, flatten)]
    pub metadata: Metadata,
    pub parameters: Vec<AdditionalParameter>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AdditionalParameter {
    pub name: String,
    pub value: Vec<Value>,
}

/// Process input description
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InputDescription {
    #[serde(flatten)]
    pub description_type: DescriptionType,
    #[serde(default = "min_occurs")]
    pub min_occurs: u64,
    #[serde(default)]
    pub max_occurs: MaxOccurs,
    pub schema: Value,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ValuePassing {
    ByValue,
    ByReference,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
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
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutputDescription {
    #[serde(flatten)]
    pub description_type: DescriptionType,
    pub schema: Value,
}

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Basic description type
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct DescriptionType {
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<Metadata>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<AdditionalParameters>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Metadata {
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AdditionalParameters {
    #[serde(default, flatten)]
    pub metadata: Metadata,
    pub parameters: Vec<AdditionalParameter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AdditionalParameter {
    pub name: String,
    pub value: Vec<Value>,
}

/// Process input description
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct InputDescription {
    #[serde(flatten)]
    pub description_type: DescriptionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_occurs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_occurs: Option<MaxOccurs>,
    pub schema: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub enum ValuePassing {
    ByValue,
    ByReference,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum MaxOccurs {
    Integer(u64),
    Unbounded(String), // FIXME: type string , enum "unbounded"
}

impl Default for MaxOccurs {
    fn default() -> Self {
        Self::Integer(1)
    }
}

/// Process output description
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct OutputDescription {
    #[serde(flatten)]
    pub description_type: DescriptionType,
    pub schema: Value,
}

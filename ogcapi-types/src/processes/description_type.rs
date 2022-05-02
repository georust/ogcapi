use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DescriptionType {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<Metadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<AdditionalParameter>,
    pub role: Option<String>,
    pub href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdditionalParameter {
    pub name: String,
    pub value: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub title: Option<String>,
    pub role: Option<String>,
    pub href: Option<String>,
}

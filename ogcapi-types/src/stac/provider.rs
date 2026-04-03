use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A provider is any of the organizations that captures or processes the content
/// of the collection and therefore influences the data offered by this collection.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
pub struct Provider {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<ProviderRole>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProviderRole {
    Licensor,
    Producer,
    Processor,
    Host,
}

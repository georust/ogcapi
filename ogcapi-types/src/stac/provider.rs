use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A provider is any of the organizations that captures or processes the content
/// of the collection and therefore influences the data offered by this collection.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
pub struct Provider {
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<Vec<ProviderRole>>,
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

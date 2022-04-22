use serde::{Deserialize, Serialize};

/// A provider is any of the organizations that captures or processes the content
/// of the collection and therefore influences the data offered by this collection.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Provider {
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<Vec<ProviderRole>>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProviderRole {
    Licensor,
    Producer,
    Processor,
    Host,
}

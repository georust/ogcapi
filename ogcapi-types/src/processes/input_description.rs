use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::DescriptionType;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
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

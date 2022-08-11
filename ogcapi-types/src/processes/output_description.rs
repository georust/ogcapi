use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::DescriptionType;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutputDescription {
    #[serde(flatten)]
    pub description_type: DescriptionType,
    pub schema: Value,
}

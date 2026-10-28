use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Queryables {
    #[serde(flatten)]
    pub queryables: HashMap<String, Queryable>,
    #[serde(rename = "additionalProperties", default = "default_true")]
    pub additional_properties: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Queryable {
    title: Option<String>,
    description: Option<String>,
    r#type: String,
}

fn default_true() -> bool {
    true
}

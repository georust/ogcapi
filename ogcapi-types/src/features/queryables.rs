use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug, Default)]
pub struct Queryables {
    #[serde(flatten)]
    pub queryables: HashMap<String, Queryable>,
    #[serde(rename = "additionalProperties", default = "default_true")]
    pub additional_properties: bool,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct Queryable {
    title: Option<String>,
    description: Option<String>,
    r#type: String,
}

fn default_true() -> bool {
    true
}

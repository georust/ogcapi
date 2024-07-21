use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Description of the property
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObservedPropertyCollection {
    /// URI linking to an external registry which contains the definitive
    /// definition of the observed property
    pub id: Option<String>,
    pub label: Label,
    pub description: Option<String>,
    pub categories: Vec<Category>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Category {
    /// URI linking to an external registry which contains the definitive
    /// definition of the observed property
    id: String,
    label: Label,
    description: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Label {
    String(String),
    Object {
        en: String,
        #[serde(flatten)]
        additional_properties: HashMap<String, String>,
    },
}

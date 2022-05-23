use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Definition of data units
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Units {
    pub id: Option<String>,
    pub label: Option<Label>,
    /// Describe unit symbol
    pub symbol: Option<Symbol>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Label {
    String(String),
    Map(HashMap<String, String>),
}

/// Describe unit symbol
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Symbol {
    String(String),
    Object {
        /// representation of the units symbol
        value: String,
        /// uri to detailed desxcription of the units
        r#type: String,
    },
}

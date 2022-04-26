use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::common::Links;

/// A STAC Catalog object represents a logical group of other `Catalog`,
/// `Collection`, and `Item` objects.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Catalog {
    /// Set to `Catalog` if this Catalog only implements the Catalog spec.
    #[serde(default = "catalog")]
    pub r#type: String,
    /// The STAC version the Catalog implements.
    #[serde(default = "crate::stac::stac_version")]
    pub stac_version: String,
    /// A list of extension identifiers the Catalog implements.
    #[serde(default)]
    pub stac_extensions: Vec<String>,
    /// Identifier for the Catalog.
    pub id: String,
    /// A short descriptive one-line title for the Catalog.
    pub title: Option<String>,
    /// Detailed multi-line description to fully explain the Catalog.
    pub description: String,
    /// A list of references to other documents.
    #[serde(default)]
    pub links: Links,
    pub conforms_to: Option<Vec<String>>,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub additional_properties: Map<String, Value>,
}

fn catalog() -> String {
    "Catalog".to_string()
}

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

use crate::common::Link;

/// A STAC Catalog object represents a logical group of other `Catalog`,
/// `Collection`, and `Item` objects.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
pub struct Catalog {
    /// Set to `Catalog` if this Catalog only implements the Catalog spec.
    #[serde(default = "crate::stac::catalog")]
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
    /// CommonMark 0.29 syntax MAY be used for rich text representation.
    #[serde(default)]
    pub description: String,
    /// A list of references to other documents.
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub additional_properties: Map<String, Value>,
}

impl Catalog {
    pub fn new(id: impl ToString, description: impl ToString) -> Self {
        Catalog {
            r#type: super::catalog(),
            stac_version: super::stac_version(),
            stac_extensions: Default::default(),
            id: id.to_string(),
            title: Default::default(),
            description: description.to_string(),
            links: Default::default(),
            additional_properties: Default::default(),
        }
    }

    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn links(mut self, links: Vec<Link>) -> Self {
        self.links = links;
        self
    }
}

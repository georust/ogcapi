use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

pub use crate::common::Collection;
#[doc(inline)]
pub use crate::features::Feature as Item;

/// A STAC Catalog object represents a logical group of other `Catalog`,
/// `Collection`, and `Item` objects.
#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Catalog {
    /// Set to `Catalog` if this Catalog only implements the Catalog spec.
    pub r#type: String,
    /// The STAC version the Catalog implements.
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
    pub links: Links,
}

/// An asset is an object that contains a link to data associated
/// with the Item that can be downloaded or streamed. It is allowed
/// to add additional fields.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Asset {
    /// URI to the asset object. Relative and absolute URI are both allowed.
    pub href: String,
    /// The displayed title for clients and users.
    pub title: Option<String>,
    /// A description of the Asset providing additional details, such as how it was processed or created.
    pub description: Option<String>,
    /// Media type of the asset.
    pub r#type: Option<String>,
    /// The semantic roles of the asset, similar to the use of rel in links.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Additional fields on the asset.
    #[serde(flatten)]
    pub additional_properties: Map<String, Value>,
}

/// A provider is any of the organizations that captures or processes the content
/// of the collection and therefore influences the data offered by this collection.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[cfg(feature = "stac")]
pub struct Provider {
    pub name: String,
    pub description: Option<String>,
    pub roles: Option<ProviderRole>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg(feature = "stac")]
pub enum ProviderRole {
    Licensor,
    Producer,
    Processor,
    Host,
}

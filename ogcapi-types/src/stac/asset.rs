use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

/// An asset is an object that contains a link to data associated
/// with the Item that can be downloaded or streamed. It is allowed
/// to add additional fields.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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
    pub roles: Option<Vec<String>>,
    /// Additional fields on the asset.
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub additional_properties: Map<String, Value>,
}

impl Asset {
    pub fn new(href: impl ToString) -> Self {
        Asset {
            href: href.to_string(),
            title: None,
            description: None,
            r#type: None,
            roles: None,
            additional_properties: Map::new(),
        }
    }
}

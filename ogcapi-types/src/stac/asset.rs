use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

/// An asset is an object that contains a link to data associated
/// with the Item that can be downloaded or streamed. It is allowed
/// to add additional fields.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    /// Additional fields on the asset.
    #[serde(flatten, default)]
    pub additional_properties: Map<String, Value>,
}

impl Asset {
    pub fn new(href: impl ToString) -> Self {
        Asset {
            href: href.to_string(),
            title: Default::default(),
            description: Default::default(),
            r#type: Default::default(),
            roles: Default::default(),
            additional_properties: Default::default(),
        }
    }

    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn description(mut self, description: impl ToString) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn media_type(mut self, media_type: impl ToString) -> Self {
        self.r#type = Some(media_type.to_string());
        self
    }

    pub fn roles(mut self, roles: &[impl ToString]) -> Self {
        self.roles = roles.iter().map(|r| r.to_string()).collect();
        self
    }
}

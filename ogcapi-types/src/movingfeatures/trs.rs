use serde::{Deserialize, Serialize};

// TODO enforce variants linkedTRS vs namedCRS

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Trs {
    r#type: String,
    properties: TrsProperties,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TrsProperties {
    r#type: Option<String>,
    name: Option<String>,
    href: Option<String>,
}

impl Default for Trs {
    fn default() -> Self {
        Self {
            r#type: "Name".to_string(),
            properties: Default::default(),
        }
    }
}

impl Default for TrsProperties {
    fn default() -> Self {
        Self {
            r#type: None,
            name: Some("urn:ogc:data:time:iso8601".to_string()),
            href: None,
        }
    }
}

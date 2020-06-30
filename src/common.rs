use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<Relation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
}

// [IANA Link Relations Registry](https://www.iana.org/assignments/link-relations/link-relations.xhtml)
//
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    Alternate,
    Collection,
    Conformance,
    Data,
    #[serde(rename = "describedBy")]
    Decribedby,
    Item,
    Items,
    License,
    Next,
    Previous,
    #[serde(rename = "self")]
    Selfie,
    ServiceDesc,
    ServiceDoc,
}

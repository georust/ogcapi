use serde::{Deserialize, Serialize};

use crate::common::ContentType;

pub type Links = Vec<Link>;

/// Hyperlink to enable Hypermedia Access
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Link {
    /// Supplies the URI to a remote resource (or resource fragment).
    pub href: String,
    /// The type or semantics of the relation.
    pub rel: LinkRelation,
    /// A hint indicating what the media type of the result of dereferencing
    /// the link should be.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ContentType>,
    /// A hint indicating what the language of the result of dereferencing the
    /// link should be.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    /// Used to label the destination of a link such that it can be used as a
    /// human-readable identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
}

/// Link Relations
///
/// [IANA Link Relations Registry](https://www.iana.org/assignments/link-relations/link-relations.xhtml)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum LinkRelation {
    Alternate,
    Collection,
    Conformance,
    Current,
    Data,
    Describedby,
    DerivedFrom,
    Exceptions,
    Execute,
    First,
    Item,
    Items,
    JobList,
    Last,
    License,
    Next,
    Parent,
    Previous,
    ProcessDesc,
    Processes,
    Results,
    Root,
    #[serde(rename = "self")]
    Selfie,
    ServiceDesc,
    ServiceDoc,
    Start,
    Status,
    Tiles,
    Up,
}

impl Default for LinkRelation {
    fn default() -> Self {
        LinkRelation::Selfie
    }
}

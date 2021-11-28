use serde::{Deserialize, Serialize};
use url::Url;

use super::MediaType;

pub type Links = Vec<Link>;

/// Hyperlink to enable Hypermedia Access
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Link {
    /// Supplies the URI to a remote resource (or resource fragment).
    #[serde(rename = "href")]
    pub url: Url,
    /// The type or semantics of the relation.
    #[serde(rename = "rel")]
    pub relation: Relation,
    /// A hint indicating what the media type of the result of dereferencing
    /// the link should be.
    #[serde(rename = "type")]
    pub mime: Option<MediaType>,
    /// A hint indicating what the language of the result of dereferencing the
    /// link should be.
    #[serde(rename = "hreflang")]
    pub language: Option<String>,
    /// Used to label the destination of a link such that it can be used as a
    /// human-readable identifier.
    pub title: Option<String>,
    pub length: Option<usize>,
}

impl Link {
    /// Constructs a new Link with the given url and the [Relation] `Self`
    pub fn new(url: Url) -> Link {
        Link {
            url: url.to_owned(),
            relation: Relation::default(),
            mime: None,
            language: None,
            title: None,
            length: None,
        }
    }

    /// Sets the [Relation] of the Link and returns the Value
    pub fn relation(mut self, relation: Relation) -> Link {
        self.relation = relation;
        self
    }

    /// Sets the [MediaType] of the Link and returns the Value
    pub fn mime(mut self, mime: MediaType) -> Link {
        self.mime = Some(mime);
        self
    }

    /// Sets the language of the Link and returns the Value
    pub fn language(mut self, language: String) -> Link {
        self.language = Some(language);
        self
    }

    /// Sets the title of the Link and returns the Value
    pub fn title(mut self, title: String) -> Link {
        self.title = Some(title);
        self
    }

    /// Sets the length of the Link and returns the Value
    pub fn length(mut self, length: usize) -> Link {
        self.length = Some(length.to_owned());
        self
    }
}

/// Link Relations
///
/// [IANA Link Relations Registry](https://www.iana.org/assignments/link-relations/link-relations.xhtml)
/// [OGC Link Relation Type Register](http://www.opengis.net/def/rel)
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    Alternate,
    Collection,
    Conformance,
    Current,
    Data,
    DataMeta,
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
    Prev,
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

impl Default for Relation {
    fn default() -> Self {
        Relation::Selfie
    }
}

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};

use super::MediaType;

pub type Links = Vec<Link>;

/// Hyperlink to enable Hypermedia Access
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Link {
    #[serde_as(as = "DisplayFromStr")]
    /// Supplies the URI to a remote resource (or resource fragment).
    pub href: String,
    /// The type or semantics of the relation.
    pub rel: String,
    /// A hint indicating what the media type of the result of dereferencing
    /// the link should be.
    pub r#type: Option<MediaType>,
    /// A hint indicating what the language of the result of dereferencing the
    /// link should be.
    pub hreflang: Option<String>,
    /// Used to label the destination of a link such that it can be used as a
    /// human-readable identifier.
    pub title: Option<String>,
    pub length: Option<usize>,
}

impl Link {
    /// Constructs a new Link with the given uri and the [LinkRel] `Self`
    pub fn new(uri: &str) -> Link {
        Link {
            href: uri.to_owned(),
            rel: LinkRel::Selfie.to_string(),
            r#type: None,
            hreflang: None,
            title: None,
            length: None,
        }
    }

    /// Sets the [LinkRel] of the Link and returns the Value
    pub fn relation(mut self, relation: LinkRel) -> Link {
        self.rel = relation.to_string();
        self
    }

    /// Sets the [MediaType] of the Link and returns the Value
    pub fn mime(mut self, mime: MediaType) -> Link {
        self.r#type = Some(mime);
        self
    }

    /// Sets the language of the Link and returns the Value
    pub fn language(mut self, language: String) -> Link {
        self.hreflang = Some(language);
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
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum LinkRel {
    About,
    /// Refers to a substitute for the link’s context.
    Alternate,
    Child,
    Collection,
    /// Refers to a resource that identifies the specifications that the link’s context conforms to.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/conformance")]
    Conformance,
    Data,
    DataMeta,
    Describedby,
    /// The target URI points to exceptions of a failed process.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/exceptions")]
    Exceptions,
    /// The target URI points to the execution endpoint of the server.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/execute")]
    Execute,
    First,
    Item,
    Items,
    /// The target URI points to the list of jobs.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/job-list")]
    JobList,
    Last,
    /// Refers to a license associated with the link’s context.
    License,
    Next,
    Parent,
    #[serde(alias = "previous")]
    Prev,
    /// The target URI points to the list of processes the API offers.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/processes")]
    Processes,
    Related,
    /// The target URI points to the results of a job.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/results")]
    Results,
    Root,
    /// Conveys an identifier for the link’s context.
    #[serde(rename = "self")]
    Selfie,
    /// Identifies service description for the context that is primarily intended for consumption by machines.
    ServiceDesc,
    /// Identifies service documentation for the context that is primarily intended for human consumption.
    ServiceDoc,
    Start,
    /// Identifies a resource that represents the context’s status.
    Status,
    Tiles,
    /// Refers to a parent document in a hierarchy of documents.
    Up,
}

impl fmt::Display for LinkRel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_value(self).unwrap().as_str().unwrap()
        )
    }
}

impl PartialEq<String> for LinkRel {
    fn eq(&self, other: &String) -> bool {
        serde_json::to_string(self).unwrap() == *other
    }
}

impl PartialEq<LinkRel> for String {
    fn eq(&self, other: &LinkRel) -> bool {
        *self == serde_json::to_value(other).unwrap().as_str().unwrap()
    }
}

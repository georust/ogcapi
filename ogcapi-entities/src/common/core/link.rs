use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

pub type Links = Vec<Link>;

/// Hyperlink to enable Hypermedia Access
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Link {
    /// Supplies the URI to a remote resource (or resource fragment).
    pub href: String,
    /// The type or semantics of the relation.
    pub rel: String,
    /// A hint indicating what the media type of the result of dereferencing
    /// the link should be.
    pub r#type: Option<String>,
    /// A hint indicating what the language of the result of dereferencing the
    /// link should be.
    pub hreflang: Option<String>,
    /// Used to label the destination of a link such that it can be used as a
    /// human-readable identifier.
    pub title: Option<String>,
    pub length: Option<i64>,
}

impl Link {
    /// Constructs a new Link with the given href and link relation
    pub fn new(href: impl ToString, rel: impl ToString) -> Link {
        Link {
            href: href.to_string(),
            rel: rel.to_string(),
            r#type: None,
            hreflang: None,
            title: None,
            length: None,
        }
    }

    /// Sets the media type of the Link and returns the Value
    pub fn mime(mut self, mime: impl ToString) -> Link {
        self.r#type = Some(mime.to_string());
        self
    }

    /// Sets the language of the Link and returns the Value
    pub fn language(mut self, language: impl ToString) -> Link {
        self.hreflang = Some(language.to_string());
        self
    }

    /// Sets the title of the Link and returns the Value
    pub fn title(mut self, title: impl ToString) -> Link {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the length of the reference resource by the Link and returns the Value
    pub fn length(mut self, length: i64) -> Link {
        self.length = Some(length);
        self
    }
}

/// Link Relations
///
/// [IANA Link Relations Registry](https://www.iana.org/assignments/link-relations/link-relations.xhtml)
/// [OGC Link Relation Type Register](http://www.opengis.net/def/rel)
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, )]
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
    /// Identifies general metadata for the context (dataset or collection) that is primarily intended for consumption by machines.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/data-meta")]
    DataMeta,
    /// Refers to a resource providing information about the link’s context.
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
    /// Identifies general metadata for the context that is primarily intended for consumption by machines.
    ServiceMeta,
    Start,
    /// Identifies a resource that represents the context’s status.
    Status,
    Tiles,
    /// The target IRI points to a resource that describes how to provide tile sets of the context resource in vector format.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/tilesets-vector")]
    TilesetsVector,
    /// The target IRI points to a resource that describes the TileMatrixSet according to the 2D-TMS standard.
    #[serde(alias = "http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme")]
    TilingScheme,
    /// Refers to a parent document in a hierarchy of documents.
    Up,
}

impl Default for LinkRel {
    fn default() -> Self {
        LinkRel::Selfie
    }
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

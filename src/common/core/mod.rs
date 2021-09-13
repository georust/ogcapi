mod bbox;
mod datetime;
mod exception;
mod link;

pub use bbox::Bbox;
pub use datetime::Datetime;
pub use exception::Exception;
pub use link::{Link, LinkRelation, Links};

use serde::{Deserialize, Serialize};

/// The Landing page is the entry point of a OGC API
///
/// The Landing page provides links to:
///
/// * the API definition (link relations service-desc and service-doc),
///
/// * the Conformance declaration (path /conformance, link relation conformance), and
///
/// * the Collections (path /collections, link relation data).
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct LandingPage {
    /// While a title is not required, implementers are strongly advised to include one.
    pub title: Option<String>,
    pub description: Option<String>,
    /// The `attribution` should be short and intended for presentation to a
    /// user, for example, in a corner of a map. Parts of the text can be links
    /// to other resources if additional information is needed. The string can
    /// include HTML markup.
    pub attribution: Option<String>,
    pub links: Links,
}

/// The Conformance declaration states the conformance classes from standards or community
/// specifications, identified by a URI, that the API conforms to. Clients can but are not
/// required to use this information. Accessing the Conformance declaration using HTTP GET
/// returns the list of URIs of conformance classes implemented by the server.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Conformance {
    pub conforms_to: Vec<String>,
}
